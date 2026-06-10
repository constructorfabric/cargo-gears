use super::{SYSTEM_REGISTRY_MODULES, SystemRegistryModule};
use crate::common::{OutputFormat, Registry};
use crate::gears_parser::{
    Capability, ConfigModule, ConfigModuleMetadata, NotFoundError, ParsedModule,
    get_module_name_from_crate, parse_module_rs_source,
};
use crate::manifest::{Manifest, ModuleRef};
use anyhow::{Context, bail};
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Display;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ModulesOutput {
    pub system: bool,
    pub local: bool,
}

impl ModulesOutput {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            system: true,
            local: true,
        }
    }

    #[must_use]
    pub const fn local() -> Self {
        Self {
            system: false,
            local: true,
        }
    }

    #[must_use]
    pub const fn system() -> Self {
        Self {
            system: true,
            local: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ModulesParams {
    pub path: Option<PathBuf>,
    pub verbose: bool,
    pub output: ModulesOutput,
    pub registry: Registry,
    pub format: OutputFormat,
}

impl ModulesParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let listing = self.collect_listing()?;

        match self.format {
            OutputFormat::Table => {
                print_modules(&listing);
                Ok(())
            }
            OutputFormat::Json => print_json_modules(&listing),
        }
    }

    fn collect_listing(&self) -> anyhow::Result<ModuleListing> {
        let should_probe_system_usage = self.output.system && self.path.is_some();
        let workspace_path = (self.output.local || should_probe_system_usage)
            .then(|| crate::common::resolve_workspace_path(self.path.as_deref()))
            .transpose()?;
        let local_modules = if self.output.local {
            Some(collect_local_module_map(workspace_path.as_deref())?)
        } else if should_probe_system_usage {
            collect_local_module_map(workspace_path.as_deref()).ok()
        } else {
            None
        };
        let manifest_system_modules = workspace_path
            .as_deref()
            .map(collect_manifest_system_modules)
            .transpose()?
            .unwrap_or_default();

        let mut modules = Vec::new();

        if self.output.system {
            modules.extend(collect_system_modules(
                self.verbose,
                self.registry,
                local_modules.as_ref(),
                &manifest_system_modules,
            )?);
        }

        if self.output.local {
            modules.extend(listed_local_modules(
                self.verbose,
                local_modules
                    .as_ref()
                    .context("local modules should be collected when local output is enabled")?,
            ));
        }

        Ok(ModuleListing {
            modules,
            output: self.output,
        })
    }
}

#[derive(Serialize)]
struct ModuleListing {
    modules: Vec<ListedModule>,
    #[serde(skip)]
    output: ModulesOutput,
}

#[derive(Serialize)]
struct ListedModule {
    name: String,
    source: ModuleSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    crate_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<ConfigModuleMetadata>,
    /// Some for a system crate, None for a local crate
    #[serde(skip_serializing_if = "Option::is_none")]
    used: Option<bool>,
    /// From the crate included
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    features: Vec<String>,
    /// Other module dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    deps: Vec<String>,
    /// From the crate included
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ModuleSource {
    System,
    Local,
}

fn collect_system_modules(
    verbose: bool,
    registry: Registry,
    local_modules: Option<&HashMap<String, ConfigModule>>,
    manifest_system_modules: &BTreeSet<String>,
) -> anyhow::Result<Vec<ListedModule>> {
    let metadata_by_crate = if verbose {
        Some(fetch_system_metadata(registry)?)
    } else {
        None
    };

    let mut modules = Vec::with_capacity(SYSTEM_REGISTRY_MODULES.len());
    for module in SYSTEM_REGISTRY_MODULES {
        let metadata = metadata_by_crate
            .as_ref()
            .map(|metadata| {
                metadata.get(module.crate_name).with_context(|| {
                    format!("missing fetched metadata for '{}'", module.crate_name)
                })
            })
            .transpose()?;
        let used = local_modules.is_some_and(|modules| modules.contains_key(module.module_name));
        let used = used
            || manifest_system_modules.contains(module.module_name)
            || manifest_system_modules.contains(module.crate_name);
        modules.push(listed_system_module(module, metadata, used));
    }

    Ok(modules)
}

fn fetch_system_metadata(
    registry: Registry,
) -> anyhow::Result<HashMap<&'static str, RegistryMetadata>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime for registry queries")?;

    runtime.block_on(fetch_all_registry_metadata(registry))
}

fn listed_system_module(
    module: &SystemRegistryModule,
    metadata: Option<&RegistryMetadata>,
    used: bool,
) -> ListedModule {
    ListedModule {
        name: module.module_name.to_owned(),
        source: ModuleSource::System,
        crate_name: Some(module.crate_name.to_owned()),
        latest_version: metadata.map(|metadata| metadata.latest_version.clone()),
        metadata: None,
        used: Some(used),
        features: metadata
            .map(|metadata| metadata.features.clone())
            .unwrap_or_default(),
        deps: metadata
            .map(|metadata| metadata.deps.clone())
            .unwrap_or_default(),
        capabilities: metadata
            .map(|metadata| {
                metadata
                    .capabilities
                    .iter()
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn collect_local_module_map(
    workspace_dir: Option<&Path>,
) -> anyhow::Result<HashMap<String, ConfigModule>> {
    get_module_name_from_crate(workspace_dir)
}

fn collect_manifest_system_modules(workspace_dir: &Path) -> anyhow::Result<BTreeSet<String>> {
    let manifest_path = workspace_dir.join(crate::manifest::DEFAULT_MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Ok(BTreeSet::new());
    }

    let manifest = Manifest::load(&manifest_path)?;
    let mut modules = BTreeSet::new();
    for envs in manifest.apps.values() {
        for environment in envs.values() {
            for module in &environment.modules {
                if let ModuleRef::Remote(remote) = module {
                    modules.insert(remote.name.clone());
                    modules.insert(remote.package.clone());
                }
            }
        }
    }

    Ok(modules)
}

fn listed_local_modules(
    verbose: bool,
    local_modules: &HashMap<String, ConfigModule>,
) -> Vec<ListedModule> {
    let mut entries: Vec<_> = local_modules.iter().collect();
    entries.sort_by_key(|(name, _)| *name);

    entries
        .into_iter()
        .map(|(module_name, module)| ListedModule {
            name: module_name.clone(),
            source: ModuleSource::Local,
            crate_name: None,
            latest_version: None,
            metadata: verbose.then(|| module.metadata.clone()),
            used: None,
            features: Vec::new(),
            deps: Vec::new(),
            capabilities: Vec::new(),
        })
        .collect()
}

fn print_modules(listing: &ModuleListing) {
    let system_modules: Vec<_> = listing
        .modules
        .iter()
        .filter(|module| module.source == ModuleSource::System)
        .collect();
    let local_modules: Vec<_> = listing
        .modules
        .iter()
        .filter(|module| module.source == ModuleSource::Local)
        .collect();
    if listing.output.system {
        if system_modules.is_empty() {
            println!("System modules:");
            println!("  (none)");
        } else {
            print_module_group("System modules:", &system_modules);
        }
    }

    if listing.output.system && listing.output.local {
        println!();
    }

    if listing.output.local {
        if local_modules.is_empty() {
            println!("Workspace modules:");
            println!("  (none)");
        } else {
            print_module_group("Workspace modules:", &local_modules);
        }
    }
}

fn print_module_group(title: &str, modules: &[&ListedModule]) {
    println!("{title}");
    for module in modules {
        print_module(module);
    }
}

fn print_module(module: &ListedModule) {
    match module.source {
        ModuleSource::System => {
            if let Some(crate_name) = &module.crate_name {
                let used_label = match module.used {
                    Some(true) => ", used: yes",
                    Some(false) => ", used: no",
                    None => "",
                };
                if module.latest_version.is_some() {
                    println!("  - {}", module.name);
                    println!("      crate: {crate_name}");
                } else {
                    println!("  - {} (crate: {crate_name}{used_label})", module.name);
                }
            }
            if module.latest_version.is_some() {
                print_optional_field("used", module.used.as_ref());
                print_optional_field("latest_version", module.latest_version.as_deref());
                print_value_list("features", &module.features);
                print_value_list("deps", &module.deps);
                print_value_list("capabilities", &module.capabilities);
            }
        }
        ModuleSource::Local => {
            println!("  - {}", module.name);
            if let Some(metadata) = &module.metadata {
                print_metadata(metadata);
            }
        }
    }
}

fn print_json_modules(listing: &ModuleListing) -> anyhow::Result<()> {
    serde_json::to_writer(std::io::stdout(), listing)?;
    println!();
    Ok(())
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
struct RegistryMetadata {
    latest_version: String,
    features: Vec<String>,
    deps: Vec<String>,
    capabilities: Vec<Capability>,
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

async fn fetch_all_registry_metadata(
    registry: Registry,
) -> anyhow::Result<HashMap<&'static str, RegistryMetadata>> {
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
    let client = Client::builder()
        .user_agent("cargo-gears")
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

    let module_metadata = fetch_gears_module_metadata(client, registry, module, &latest_version)
        .await
        .with_context(|| {
            format!(
                "failed to find gears module in src/ for {}",
                module.crate_name
            )
        })?;

    Ok(RegistryMetadata {
        latest_version,
        features,
        deps: module_metadata.deps,
        capabilities: module_metadata.capabilities,
    })
}

async fn fetch_gears_module_metadata(
    client: &Client,
    registry: Registry,
    module: SystemRegistryModule,
    latest_version: &str,
) -> anyhow::Result<ParsedModule> {
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

    extract_gears_module(crate_archive.as_ref())
}

/// Scans all `.rs` files directly under `src/` in the crate archive for a gears
/// module annotation, returning the parsed metadata from the first match.
fn extract_gears_module(crate_archive: &[u8]) -> anyhow::Result<ParsedModule> {
    let decoder = GzDecoder::new(Cursor::new(crate_archive));
    let mut archive = tar::Archive::new(decoder);
    let entries = archive
        .entries()
        .context("failed to list crate archive entries")?;

    for entry in entries {
        let mut entry = entry.context("failed to read crate archive entry")?;
        let path = entry
            .path()
            .context("failed to read crate archive entry path")?
            .into_owned();
        if is_src_rs_entry(&path) {
            let mut content = String::new();
            entry
                .read_to_string(&mut content)
                .context("failed to read .rs file from crate archive")?;
            match parse_module_rs_source(&content) {
                Ok(parsed) => return Ok(parsed),
                Err(e) if e.is::<NotFoundError>() => {}
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("failed to parse {} from crate archive", path.display())
                    });
                }
            }
        }
    }

    bail!("crate archive does not contain a gears module annotation in src/")
}

/// Returns `true` if the archive entry is a `.rs` file under `src/` at any depth.
///
/// Archive entries look like `<crate-version>/src/<file>.rs` or
/// `<crate-version>/src/subdir/<file>.rs`.
/// We check that the file has a `.rs` extension and has `src` as an ancestor.
fn is_src_rs_entry(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "rs")
        && path
            .ancestors()
            .any(|a| a.file_name().is_some_and(|name| name == "src"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn scaffold_workspace(modules: &[(&str, &str)]) -> TempDir {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        let members: Vec<_> = modules
            .iter()
            .map(|(crate_name, _)| format!(r#""{crate_name}""#))
            .collect();
        let members_str = members.join(", ");

        write_file(
            temp_dir.path().join("Cargo.toml"),
            &format!(
                r#"
                [workspace]
                members = [{members_str}]
                resolver = "3"
                "#,
            ),
        );

        for (crate_name, module_name) in modules {
            write_file(
                temp_dir.path().join(crate_name).join("Cargo.toml"),
                &format!(
                    r#"
                    [package]
                    name = "{crate_name}"
                    version = "0.1.0"
                    edition = "2024"

                    [lib]
                    path = "src/lib.rs"
                    "#,
                ),
            );
            write_file(
                temp_dir.path().join(crate_name).join("src/lib.rs"),
                "pub mod module;",
            );
            write_file(
                temp_dir.path().join(crate_name).join("src/module.rs"),
                &format!(
                    r#"
                    #[module(name = "{module_name}")]
                    pub struct Module;
                    "#,
                ),
            );
        }

        temp_dir
    }

    fn write_file(path: PathBuf, contents: &str) {
        fs::create_dir_all(path.parent().expect("test path should have parent"))
            .expect("failed to create parent directory");
        fs::write(path, contents).expect("failed to write test file");
    }

    #[test]
    fn is_src_rs_entry_matches_rs_file_under_src() {
        assert!(is_src_rs_entry(Path::new("crate-0.1.0/src/module.rs")));
        assert!(is_src_rs_entry(Path::new("crate-0.1.0/src/lib.rs")));
        assert!(is_src_rs_entry(Path::new("crate-0.1.0/src/gear.rs")));
    }

    #[test]
    fn is_src_rs_entry_rejects_non_rs_files() {
        assert!(!is_src_rs_entry(Path::new("crate-0.1.0/src/Cargo.toml")));
        assert!(!is_src_rs_entry(Path::new("crate-0.1.0/src/README.md")));
    }

    #[test]
    fn is_src_rs_entry_matches_nested_rs_files() {
        assert!(is_src_rs_entry(Path::new("crate-0.1.0/src/api/handler.rs")));
        assert!(is_src_rs_entry(Path::new(
            "crate-0.1.0/src/domain/model.rs"
        )));
        assert!(is_src_rs_entry(Path::new("crate-0.1.0/src/a/b/c/deep.rs")));
    }

    #[test]
    fn is_src_rs_entry_rejects_rs_outside_src() {
        assert!(!is_src_rs_entry(Path::new("crate-0.1.0/tests/test.rs")));
        assert!(!is_src_rs_entry(Path::new("crate-0.1.0/benches/bench.rs")));
    }

    #[test]
    fn local_json_includes_verbose_metadata() {
        let temp_dir = scaffold_workspace(&[("crate-echo", "echo")]);
        let args = ModulesParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: true,
            output: ModulesOutput::local(),
            registry: Registry::CratesIo,
            format: OutputFormat::Json,
        };

        let listing = args
            .collect_listing()
            .expect("module collection should succeed");
        let value = serde_json::to_value(&listing).expect("modules should serialize as JSON");

        assert_eq!(value["modules"][0]["name"], json!("echo"));
        assert_eq!(value["modules"][0]["source"], json!("local"));
        assert_eq!(
            value["modules"][0]["metadata"]["package"],
            json!("crate-echo")
        );
        assert_eq!(value["modules"][0]["metadata"]["version"], json!("0.1.0"));
        assert!(
            value["modules"][0]["metadata"]["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("crate-echo"))
        );
    }

    #[test]
    fn system_json_includes_static_registry_without_verbose_fetch() {
        let args = ModulesParams {
            path: None,
            verbose: false,
            output: ModulesOutput::system(),
            registry: Registry::CratesIo,
            format: OutputFormat::Json,
        };

        let listing = args
            .collect_listing()
            .expect("module collection should succeed");
        let value = serde_json::to_value(&listing).expect("modules should serialize as JSON");
        let first_module = &value["modules"][0];

        assert_eq!(listing.modules.len(), SYSTEM_REGISTRY_MODULES.len());
        assert_eq!(first_module["source"], "system");
        assert_eq!(first_module["name"], SYSTEM_REGISTRY_MODULES[0].module_name);
        assert_eq!(
            first_module["crate_name"],
            SYSTEM_REGISTRY_MODULES[0].crate_name
        );
        assert_eq!(first_module["used"], false);
        assert!(first_module.get("latest_version").is_none());
    }

    #[test]
    fn system_json_marks_system_module_used_when_workspace_has_matching_module() {
        let temp_dir = scaffold_workspace(&[("crate-credstore", "credstore")]);
        let args = ModulesParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: false,
            output: ModulesOutput::system(),
            registry: Registry::CratesIo,
            format: OutputFormat::Json,
        };

        let listing = args
            .collect_listing()
            .expect("module collection should succeed");
        let value = serde_json::to_value(&listing).expect("modules should serialize as JSON");
        let credstore = value["modules"]
            .as_array()
            .expect("modules should be an array")
            .iter()
            .find(|module| module["name"] == "credstore")
            .expect("credstore should be listed");

        assert_eq!(credstore["used"], true);
    }

    #[test]
    fn system_json_marks_system_module_used_when_manifest_references_remote_module() {
        let temp_dir = scaffold_workspace(&[]);
        write_file(
            temp_dir.path().join("Gears.toml"),
            r#"
            [workspace]
            version = 1

            [apps.quickstart.dev]
            config = "quickstart.yml"
            modules = [
                { source = "remote", name = "api-gateway", package = "cf-api-gateway", version = "0.2.7" },
            ]
            "#,
        );
        let args = ModulesParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: false,
            output: ModulesOutput::system(),
            registry: Registry::CratesIo,
            format: OutputFormat::Json,
        };

        let listing = args
            .collect_listing()
            .expect("module collection should succeed");
        let value = serde_json::to_value(&listing).expect("modules should serialize as JSON");
        let api_gateway = value["modules"]
            .as_array()
            .expect("modules should be an array")
            .iter()
            .find(|module| module["name"] == "api-gateway")
            .expect("api-gateway should be listed");

        assert_eq!(api_gateway["used"], true);
    }
}
