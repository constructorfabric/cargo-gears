use super::{SYSTEM_REGISTRY_MODULES, SystemRegistryModule, load_config};
use crate::app_config::ModuleConfig;
use crate::common::{PathConfigArgs, Registry};
use crate::module_parser::{
    Capability, ConfigModule, ConfigModuleMetadata, get_module_name_from_crate,
    parse_module_rs_source,
};
use anyhow::{Context, bail};
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Display;
use std::io::{Cursor, Read};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Eq, PartialEq)]
pub struct ListArgs {
    pub path_config: PathConfigArgs,
    /// Show system crates also. If verbose is enabled,
    /// fetches registry metadata for system crates. (makes requests to the registry)
    pub system: bool,
    /// Show all information related to the module.
    pub verbose: bool,
    /// Registry to query for system-crate metadata. Only consulted when both
    /// `--system` and `--verbose` are enabled; `--verbose` alone does not query
    /// any registry. Defaults to `crates.io`.
    pub registry: Registry,
}

impl ListArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.path_config.with_workspace_dir(|config_path| {
            let local_modules = get_module_name_from_crate()?;
            let config = load_config(config_path)?;
            let enabled_modules: BTreeSet<_> = config.modules.keys().map(String::as_str).collect();

            if self.system {
                println!("System crates:");
                if self.verbose {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .context("failed to build tokio runtime for registry queries")?;

                    let metadata_by_crate =
                        runtime.block_on(fetch_all_registry_metadata(self.registry))?;

                    for module in SYSTEM_REGISTRY_MODULES {
                        let Some(metadata) = metadata_by_crate.get(module.crate_name) else {
                            bail!("missing fetched metadata for '{}'", module.crate_name);
                        };

                        print_system_registry_metadata(module, metadata);
                    }
                } else {
                    for module in SYSTEM_REGISTRY_MODULES {
                        println!("  - {}", module.module_name);
                    }
                }
            }

            println!();
            println!("Workspace modules:");
            if local_modules.is_empty() {
                println!("  (none)");
            } else {
                let mut local_entries: Vec<_> = local_modules.iter().collect();
                local_entries.sort_by_key(|(left_name, _)| *left_name);
                for (module_name, module) in local_entries {
                    let enabled_label = if enabled_modules.contains(module_name.as_str()) {
                        " (enabled in config)"
                    } else {
                        ""
                    };
                    println!("  - {module_name}{enabled_label}");

                    if self.verbose {
                        print_local_metadata(module);
                    }
                }
            }

            println!();
            println!("Modules enabled in config ({}):", config_path.display());
            if config.modules.is_empty() {
                println!("  (none)");
            } else {
                let mut configured_entries: Vec<_> = config.modules.iter().collect();
                configured_entries.sort_by_key(|(left_name, _)| *left_name);
                for (module_name, module) in configured_entries {
                    let location_label = if local_modules.contains_key(module_name.as_str()) {
                        " (local workspace)"
                    } else {
                        " (not found in workspace)"
                    };
                    println!("  - {module_name}{location_label}");

                    if self.verbose {
                        print_config_metadata(module);
                    }
                }
            }

            Ok(())
        })
    }
}

fn print_local_metadata(module: &ConfigModule) {
    print_metadata(&module.metadata);
}

fn print_config_metadata(module: &ModuleConfig) {
    let Some(metadata) = &module.metadata else {
        println!("      metadata: (none)");
        return;
    };

    print_metadata(metadata);
}

fn print_system_registry_metadata(module: &SystemRegistryModule, metadata: &RegistryMetadata) {
    println!("  - {}", module.module_name);
    println!("      crate: {}", module.crate_name);
    println!("      latest_version: {}", metadata.latest_version);
    print_value_list("features", &metadata.features);
    print_value_list("deps", &metadata.deps);
    print_value_list("capabilities", &metadata.capabilities);
}

fn print_metadata(metadata: &ConfigModuleMetadata) {
    print_optional_field("package", metadata.package.as_deref());
    print_optional_field("version", metadata.version.as_deref());
    print_optional_field("path", metadata.path.as_deref());
    print_optional_field("default_features", metadata.default_features.as_ref());

    print_value_list("features", &metadata.features);
    print_value_list("deps", &metadata.deps);
    print_value_list("capabilities", &metadata.capabilities);
}

fn print_optional_field<T: Display>(label: &str, value: Option<T>) {
    if let Some(value) = value {
        println!("      {label}: {value}");
    }
}

fn print_value_list<T: Display>(label: &str, values: &[T]) {
    if values.is_empty() {
        println!("      {label}: (none)");
    } else {
        println!("      {label}:");
        for value in values {
            println!("        - {value}");
        }
    }
}

#[derive(Default)]
pub(crate) struct RegistryMetadata {
    pub(crate) latest_version: String,
    pub(crate) features: Vec<String>,
    pub(crate) deps: Vec<String>,
    pub(crate) capabilities: Vec<Capability>,
}

#[derive(Deserialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
    versions: Vec<CrateVersion>,
}

#[derive(Deserialize)]
struct CrateInfo {
    max_version: String,
}

#[derive(Deserialize)]
struct CrateVersion {
    num: String,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
}

pub(crate) async fn fetch_all_registry_metadata(
    registry: Registry,
) -> anyhow::Result<HashMap<&'static str, RegistryMetadata>> {
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
    let client = Client::builder()
        .user_agent("cyberfabric-cli")
        .timeout(Duration::from_secs(10))
        .build()
        .context("failed to create registry HTTP client")?;

    let mut join_set = tokio::task::JoinSet::new();
    for module in SYSTEM_REGISTRY_MODULES.iter().copied() {
        let cloned_client = client.clone();
        let permit_pool = semaphore.clone();
        join_set.spawn(async move {
            let _permit = permit_pool
                .acquire_owned()
                .await
                .context("failed to acquire registry fetch permit")?;
            let metadata = fetch_registry_metadata(&cloned_client, registry, module)
                .await
                .with_context(|| format!("failed to fetch metadata for '{}'", module.crate_name))?;
            Ok::<_, anyhow::Error>((module.crate_name, metadata))
        });
    }

    let mut metadata_by_crate = HashMap::with_capacity(join_set.len());
    while let Some(task_result) = join_set.join_next().await {
        let (crate_name, metadata) = task_result.context("registry task panicked")??;
        metadata_by_crate.insert(crate_name, metadata);
    }

    Ok(metadata_by_crate)
}

async fn fetch_registry_metadata(
    client: &Client,
    registry: Registry,
    module: SystemRegistryModule,
) -> anyhow::Result<RegistryMetadata> {
    let crate_url = format!("https://{registry}/api/v1/crates/{}", module.crate_name);
    let crate_response = client
        .get(&crate_url)
        .send()
        .await
        .with_context(|| format!("request failed for {}", module.crate_name))?
        .error_for_status()
        .with_context(|| format!("registry returned an error for {}", module.crate_name))?
        .json::<CrateResponse>()
        .await
        .with_context(|| format!("invalid crate metadata for {}", module.crate_name))?;

    let latest_version = crate_response.crate_info.max_version;
    let features = crate_response
        .versions
        .into_iter()
        .find(|version| version.num == latest_version)
        .map_or_else(Vec::new, |version| version.features.into_keys().collect());

    let module_rs_content =
        fetch_module_rs_content(client, registry, module, &latest_version).await?;
    let module_metadata = parse_module_rs_source(&module_rs_content)
        .with_context(|| format!("invalid src/module.rs for {}", module.crate_name))?;

    Ok(RegistryMetadata {
        latest_version,
        features,
        deps: module_metadata.deps,
        capabilities: module_metadata.capabilities,
    })
}

async fn fetch_module_rs_content(
    client: &Client,
    registry: Registry,
    module: SystemRegistryModule,
    latest_version: &str,
) -> anyhow::Result<String> {
    let download_url = format!(
        "https://{registry}/api/v1/crates/{}/{}/download",
        module.crate_name, latest_version
    );
    let crate_archive = client
        .get(&download_url)
        .send()
        .await
        .with_context(|| format!("download request failed for {}", module.crate_name))?
        .error_for_status()
        .with_context(|| {
            format!(
                "download endpoint returned an error for {}",
                module.crate_name
            )
        })?
        .bytes()
        .await
        .with_context(|| format!("failed to read downloaded source for {}", module.crate_name))?;

    extract_module_rs(crate_archive.as_ref())
        .with_context(|| format!("failed to extract src/module.rs for {}", module.crate_name))
}

fn extract_module_rs(crate_archive: &[u8]) -> anyhow::Result<String> {
    let decoder = GzDecoder::new(Cursor::new(crate_archive));
    let mut archive = tar::Archive::new(decoder);
    let entries = archive
        .entries()
        .context("failed to list crate archive entries")?;

    for entry in entries {
        let mut entry = entry.context("failed to read crate archive entry")?;
        let path = entry
            .path()
            .context("failed to read crate archive entry path")?;
        if path.ends_with(Path::new("src/module.rs")) {
            let mut module_rs = String::new();
            entry
                .read_to_string(&mut module_rs)
                .context("failed to read src/module.rs from crate archive")?;
            return Ok(module_rs);
        }
    }

    bail!("crate archive does not contain src/module.rs")
}
