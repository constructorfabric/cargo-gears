use crate::app_config::AppConfig;
use crate::config::validate_name;
use crate::module_parser::{
    CargoToml, CargoTomlDependencies, CargoTomlDependency, ConfigModuleMetadata, Package,
    get_dependencies, get_module_name_from_crate,
};
use anyhow::Context;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

#[derive(Debug, Eq, PartialEq)]
pub struct PathConfigArgs {
    /// Path to the module workspace root
    pub path: Option<PathBuf>,
    /// Path to the config file
    pub config: PathBuf,
}

pub fn parse_and_chdir(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);

    if !path.is_dir() {
        return Err(format!("not a directory: {}", path.display()));
    }

    let path = path
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize {}: {e}", path.display()))?;
    env::set_current_dir(&path)
        .map_err(|e| format!("failed to change directory to {}: {e}", path.display()))?;

    Ok(path)
}

impl PathConfigArgs {
    pub fn resolve_config(&self) -> anyhow::Result<PathBuf> {
        let workspace_path = resolve_workspace_path(self.path.as_deref())?;
        resolve_config_from_workspace(&workspace_path, &self.config)
    }

    pub fn with_workspace_dir<T>(
        &self,
        f: impl FnOnce(&Path) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        let workspace_path = resolve_workspace_path(self.path.as_deref())?;
        let config_path = resolve_config_from_workspace(&workspace_path, &self.config)?;
        with_current_dir(&workspace_path, || f(&config_path))
    }
}

pub fn resolve_workspace_path(path: Option<&Path>) -> anyhow::Result<PathBuf> {
    path.map_or_else(workspace_root, |path| {
        path.canonicalize()
            .with_context(|| format!("can't canonicalize workspace path {}", path.display()))
    })
}

fn resolve_config_from_workspace(workspace_path: &Path, config: &Path) -> anyhow::Result<PathBuf> {
    let config_path = if config.is_absolute() {
        config.to_path_buf()
    } else {
        workspace_path.join(config)
    };

    config_path
        .canonicalize()
        .with_context(|| format!("can't canonicalize config {}", config_path.display()))
}

pub fn with_current_dir<T>(
    path: &Path,
    f: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    let original_dir = env::current_dir().context("can't determine current working directory")?;
    env::set_current_dir(path)
        .with_context(|| format!("failed to change directory to {}", path.display()))?;

    let result = f();
    let restore_result = env::set_current_dir(&original_dir).with_context(|| {
        format!(
            "failed to restore current directory to {}",
            original_dir.display()
        )
    });

    restore_result?;
    result
}

pub fn with_current_dir_for_optional_path<T>(
    path: Option<&Path>,
    f: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    match path {
        Some(path) => {
            let workspace_path = resolve_workspace_path(Some(path))?;
            with_current_dir(&workspace_path, f)
        }
        None => f(),
    }
}

pub fn workspace_root() -> anyhow::Result<PathBuf> {
    env::current_dir().context("can't determine current working directory")
}

#[derive(Debug, Eq, PartialEq)]
pub struct BuildRunArgs {
    pub path_config: PathConfigArgs,
    /// Use OpenTelemetry tracing
    pub otel: bool,
    /// Enable FIPS mode
    pub fips: bool,
    /// Build/run in release mode
    pub release: bool,
    /// Remove Cargo.lock at the start of the execution
    pub clean: bool,
    /// Override the generated server and binary name
    pub name: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Registry {
    #[default]
    #[cfg_attr(feature = "clap", value(name = "crates.io"))]
    CratesIo,
}

impl Registry {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CratesIo => "crates.io",
        }
    }
}

impl Display for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
    Toml,
}

impl BuildRunArgs {
    pub fn resolve_config_and_name(&self) -> anyhow::Result<(PathBuf, String)> {
        self.path_config.with_workspace_dir(|config_path| {
            let project_name = resolve_generated_project_name(config_path, self.name.as_deref())?;
            if self.clean {
                remove_from_file_structure(&project_name, "Cargo.lock")?;
            }

            Ok((config_path.to_path_buf(), project_name))
        })
    }
}

pub const BASE_PATH: &str = ".cyberfabric";

const CONFIG_PATH_ENV_VAR: &str = "CF_CLI_CONFIG";

const CARGO_CONFIG_TOML: &str = r#"[build]
target-dir = "../../target"
build-dir = "../../target"
"#;

const CARGO_SERVER_MAIN: &str = r#"
use anyhow::{Context, Result};
use modkit::bootstrap::{AppConfig, /* run_migrate, */ run_server};
{{dependencies}}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = std::env::var_os("CF_CLI_CONFIG")
        .map(std::path::PathBuf::from)
        .context("CF_CLI_CONFIG is not set")?;
    let config = AppConfig::load_or_default(Some(&config_path))?;

    run_server(config).await
}"#;

/// Returns the `cargo` binary path from the `CARGO` environment variable.
///
/// This variable is set automatically by Cargo when running subcommands.
/// Because the CLI is only distributed as `cargo cyberfabric`, this variable
/// must always be present; if it is missing we return an error instead of
/// silently falling back.
pub fn cargo_cmd() -> anyhow::Result<Command> {
    env::var_os("CARGO").context(
        "CARGO environment variable is not set — the CLI must be invoked as `cargo cyberfabric`",
    ).map(Command::new)
}

pub fn cargo_command(
    subcommand: &str,
    path: &Path,
    config_path: &Path,
    otel: bool,
    fips: bool,
    release: bool,
) -> anyhow::Result<Command> {
    let mut cmd = cargo_cmd()?;
    cmd.arg(subcommand);
    cmd.env(CONFIG_PATH_ENV_VAR, config_path.as_os_str());
    if otel {
        cmd.arg("-F").arg("otel");
    }
    if fips {
        cmd.arg("-F").arg("fips");
    }
    if release {
        cmd.arg("-r");
    }
    cmd.current_dir(path);
    Ok(cmd)
}

pub fn get_config(config_path: &Path) -> anyhow::Result<AppConfig> {
    let mut config = get_config_from_path(config_path)?;
    let mut members = get_module_name_from_crate()?;

    config.modules.iter_mut().for_each(|module| {
        if let Some(module_metadata) = members.remove(module.0.as_str()) {
            let config_metadata = std::mem::take(&mut module.1.metadata).unwrap_or_default();
            module.1.metadata = Some(merge_module_metadata(
                config_metadata,
                module_metadata.metadata,
            ));
        } else {
            eprintln!(
                "info: config module '{}' not found locally, retrieving it from the registry",
                module.0
            );
        }
    });

    Ok(config)
}

fn get_config_from_path(path: &Path) -> anyhow::Result<AppConfig> {
    let config = fs::File::open(path).context("config not available")?;
    serde_saphyr::from_reader(config).context("config not valid")
}

fn merge_module_metadata(
    config_metadata: ConfigModuleMetadata,
    local_metadata: ConfigModuleMetadata,
) -> ConfigModuleMetadata {
    let features = if config_metadata.features.is_empty() {
        local_metadata.features
    } else {
        config_metadata.features
    };

    ConfigModuleMetadata {
        package: config_metadata.package.or(local_metadata.package),
        version: config_metadata.version.or(local_metadata.version),
        features,
        default_features: config_metadata
            .default_features
            .or(local_metadata.default_features),
        path: config_metadata.path.or(local_metadata.path),
        deps: local_metadata.deps,
        capabilities: local_metadata.capabilities,
    }
}

static FEATURES: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut res = HashMap::with_capacity(2);
    res.insert("default".to_owned(), vec![]);
    res.insert("otel".to_owned(), vec!["modkit/otel".to_owned()]);
    res.insert("fips".to_owned(), vec!["modkit/fips".to_owned()]);
    res
});

static CARGO_DEPS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut res = HashMap::with_capacity(5);
    res.insert("cf-modkit".to_owned(), "modkit".to_owned());
    res.insert("modkit".to_owned(), "modkit".to_owned()); // just in case there's a renamed
    res.insert("anyhow".to_owned(), "anyhow".to_owned());
    res.insert("tokio".to_owned(), "tokio".to_owned());
    res
});

fn create_required_deps() -> anyhow::Result<CargoTomlDependencies> {
    let workspace_path = workspace_root()?;
    let mut deps = get_dependencies(&workspace_path, &CARGO_DEPS)?;
    if let Some(modkit) = deps.get_mut("modkit") {
        modkit.features.insert("bootstrap".to_owned());
    } else {
        deps.insert(
            "modkit".to_owned(),
            CargoTomlDependency {
                package: Some("cf-modkit".to_owned()),
                features: BTreeSet::from(["bootstrap".to_owned()]),
                ..Default::default()
            },
        );
    }
    if let Some(tokio) = deps.get_mut("tokio") {
        tokio.features.insert("full".to_owned());
    } else {
        deps.insert(
            "tokio".to_owned(),
            CargoTomlDependency {
                features: BTreeSet::from(["full".to_owned()]),
                version: Some("1".to_owned()),
                ..Default::default()
            },
        );
    }
    Ok(deps)
}

pub fn generate_server_structure(
    project_name: &str,
    current_dependencies: &CargoTomlDependencies,
) -> anyhow::Result<()> {
    let workspace = workspace_root()?
        .to_str()
        .context("workspace path is not valid UTF-8")?
        .to_owned();
    let mut dependencies: CargoTomlDependencies = current_dependencies
        .iter()
        .map(|(name, dep)| (name.clone(), make_absolute_paths_relative(dep, &workspace)))
        .collect();
    dependencies.extend(create_required_deps()?);
    let cargo_toml = CargoToml {
        package: Package {
            name: project_name.to_owned(),
            ..Default::default()
        },
        dependencies,
        features: FEATURES.clone(),
        ..Default::default()
    };
    let cargo_toml_str =
        toml::to_string(&cargo_toml).context("something went wrong when transforming to toml")?;
    let main_rs = prepare_cargo_server_main(current_dependencies);

    create_file_structure(project_name, "Cargo.toml", &cargo_toml_str)?;
    create_file_structure(project_name, ".cargo/config.toml", CARGO_CONFIG_TOML)?;
    create_file_structure(project_name, "src/main.rs", &main_rs)?;

    Ok(())
}

// Transforms absolute paths into relative paths, ugly but works
fn make_absolute_paths_relative(dep: &CargoTomlDependency, workspace: &str) -> CargoTomlDependency {
    let mut dep = dep.clone();
    if let Some(path) = &dep.path {
        let workspace_path = Path::new(workspace);
        let dependency_path = Path::new(path);
        let stripped = if dependency_path.is_absolute() {
            dependency_path
                .strip_prefix(workspace_path)
                .ok()
                .map(Path::to_path_buf)
                .or_else(|| {
                    let workspace_path = workspace_path.canonicalize().ok()?;
                    let dependency_path = dependency_path.canonicalize().ok()?;
                    dependency_path
                        .strip_prefix(&workspace_path)
                        .ok()
                        .map(Path::to_path_buf)
                })
        } else {
            // Workspace-relative paths are written relative to the workspace
            // root, so they need the same ../.. prefix as stripped absolute
            // paths when rewritten into the generated project.
            Some(dependency_path.to_path_buf())
        };

        if let Some(stripped) = stripped {
            dep.path = Some(
                Path::new("../..")
                    .join(stripped)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    dep
}

pub fn generated_project_dir(project_name: &str) -> anyhow::Result<PathBuf> {
    Ok(workspace_root()?.join(BASE_PATH).join(project_name))
}

fn create_file_structure(
    project_name: &str,
    relative_path: &str,
    contents: &str,
) -> anyhow::Result<()> {
    use std::io::Write;
    let path = generated_project_dir(project_name)?.join(relative_path);
    fs::create_dir_all(
        path.parent().context(
            "this should be unreachable, the parent for the file structure always exists",
        )?,
    )
    .context("can't create directory")?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .context("can't create file")?;
    file.write_all(contents.as_bytes())
        .context("can't write to file")
}

pub(crate) fn remove_from_file_structure(
    project_name: &str,
    relative_path: &str,
) -> anyhow::Result<()> {
    let path = generated_project_dir(project_name)?.join(relative_path);
    if path.exists() {
        fs::remove_file(path).context("can't remove file")?;
    }
    Ok(())
}

pub fn resolve_generated_project_name(
    config_path: &Path,
    override_name: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(name) = override_name {
        validate_name(name, "server")?;
        return Ok(name.to_owned());
    }

    let file_stem = config_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("config filename is not valid UTF-8")?;
    validate_name(file_stem, "server").with_context(|| {
        format!(
            "invalid generated server name '{file_stem}' from config file {}; use --name to override",
            config_path.display()
        )
    })?;

    Ok(file_stem.to_owned())
}

fn prepare_cargo_server_main(dependencies: &CargoTomlDependencies) -> String {
    use std::fmt::Write;

    let dependencies = dependencies.keys().fold(String::new(), |mut acc, name| {
        let rust_name = name.replace('-', "_");
        _ = writeln!(acc, "use {rust_name} as _;");
        acc
    });

    CARGO_SERVER_MAIN.replace("{{dependencies}}", &dependencies)
}

#[cfg(test)]
mod tests {
    use super::{
        cargo_command, generate_server_structure, generated_project_dir,
        make_absolute_paths_relative, merge_module_metadata, prepare_cargo_server_main,
        resolve_generated_project_name,
    };
    use crate::module_parser::{
        Capability, CargoTomlDependencies, CargoTomlDependency, ConfigModuleMetadata,
        test_utils::{CWD_MUTEX, TempDirExt},
    };
    use std::env;
    use std::path::Path;
    use tempfile::TempDir;

    struct CwdRestoreGuard {
        original_dir: std::path::PathBuf,
    }

    impl Drop for CwdRestoreGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original_dir);
        }
    }

    fn write_package(temp_dir: &TempDir, relative_path: &str, package_name: &str) {
        temp_dir.write(
            &format!("{relative_path}/Cargo.toml"),
            &format!(
                r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"
"#
            ),
        );
        temp_dir.write(
            &format!("{relative_path}/src/lib.rs"),
            "pub fn marker() {}\n",
        );
    }

    #[test]
    fn merge_module_metadata_preserves_config_overrides() {
        let config_metadata = ConfigModuleMetadata {
            package: None,
            version: None,
            features: vec!["grpc".to_owned(), "otel".to_owned()],
            default_features: Some(false),
            path: Some("modules/custom-path".to_owned()),
            deps: vec![],
            capabilities: vec![],
        };
        let local_metadata = ConfigModuleMetadata {
            package: Some("cf-demo".to_owned()),
            version: Some("0.5.0".to_owned()),
            features: vec![],
            default_features: None,
            path: Some("modules/demo".to_owned()),
            deps: vec!["authz".to_owned()],
            capabilities: vec![Capability::Grpc],
        };

        let merged = merge_module_metadata(config_metadata, local_metadata);
        assert_eq!(merged.package.as_deref(), Some("cf-demo"));
        assert_eq!(merged.version.as_deref(), Some("0.5.0"));
        assert_eq!(merged.features, vec!["grpc", "otel"]);
        assert_eq!(merged.default_features, Some(false));
        assert_eq!(merged.path.as_deref(), Some("modules/custom-path"));
        assert_eq!(merged.deps, vec!["authz"]);
        assert_eq!(merged.capabilities, vec![Capability::Grpc]);
    }

    #[test]
    fn generated_project_name_defaults_to_config_file_stem() {
        let name = resolve_generated_project_name(Path::new("/tmp/quickstart.yml"), None)
            .expect("config stem should resolve to a project name");

        assert_eq!(name, "quickstart");
    }

    #[test]
    fn generated_project_name_prefers_explicit_override() {
        let name = resolve_generated_project_name(Path::new("/tmp/quickstart.yml"), Some("demo"))
            .expect("explicit override should resolve to a project name");

        assert_eq!(name, "demo");
    }

    #[test]
    fn generated_server_main_reads_config_from_env_and_includes_dependencies() {
        let dependencies = CargoTomlDependencies::from([
            ("module_a".to_owned(), CargoTomlDependency::default()),
            ("module_b".to_owned(), CargoTomlDependency::default()),
            ("api-db-handler".to_owned(), CargoTomlDependency::default()),
        ]);

        let main_rs = prepare_cargo_server_main(&dependencies);

        assert!(main_rs.contains("std::env::var_os(\"CF_CLI_CONFIG\")"));
        assert!(main_rs.contains("use module_a as _;"));
        assert!(main_rs.contains("use module_b as _;"));
        assert!(main_rs.contains("use api_db_handler as _;"));
        assert!(!main_rs.contains("use api-db-handler as _;"));
        assert!(!main_rs.contains("{{dependencies}}"));
    }

    #[test]
    fn cargo_command_passes_selected_generated_project_features() {
        let config_path = Path::new("/tmp/config.yml");
        let cargo_dir = Path::new("/tmp/generated");

        // CARGO env var is set by `cargo test`, so cargo_command succeeds.
        let command = cargo_command("run", cargo_dir, config_path, true, true, true)
            .expect("cargo_command should succeed when CARGO is set");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(args, vec!["run", "-F", "otel", "-F", "fips", "-r"]);
        assert_eq!(command.get_current_dir(), Some(cargo_dir));
    }

    #[test]
    fn make_absolute_paths_relative_rewrites_workspace_paths() {
        let dependency = CargoTomlDependency {
            path: Some("/tmp/workspace/crates/local-module".to_owned()),
            ..Default::default()
        };

        let rewritten = make_absolute_paths_relative(&dependency, "/tmp/workspace");
        let rewritten_path = Path::new(
            rewritten
                .path
                .as_deref()
                .expect("rewritten dependency should keep a path"),
        );

        assert!(!rewritten_path.is_absolute());
        assert!(rewritten_path.is_relative());
        assert_eq!(rewritten.path.as_deref(), Some("../../crates/local-module"));
    }

    #[test]
    fn make_absolute_paths_relative_rewrites_workspace_relative_paths() {
        let dependency = CargoTomlDependency {
            path: Some("crates/local-module".to_owned()),
            ..Default::default()
        };

        let rewritten = make_absolute_paths_relative(&dependency, "/tmp/workspace");

        assert_eq!(rewritten.path.as_deref(), Some("../../crates/local-module"));
    }

    #[test]
    fn generate_server_structure_writes_existing_relative_dependency_paths() {
        let _guard = CWD_MUTEX
            .lock()
            .expect("current-dir test lock should not be poisoned");
        let _cwd_guard = CwdRestoreGuard {
            original_dir: env::current_dir().expect("current dir should be available"),
        };
        let temp_dir = TempDir::new().expect("temp dir should be created");

        write_package(&temp_dir, "crates/anyhow", "anyhow");
        write_package(&temp_dir, "crates/tokio", "tokio");
        write_package(&temp_dir, "crates/modkit", "cf-modkit");
        write_package(&temp_dir, "crates/local-module", "local-module");
        temp_dir.write(
            "Cargo.toml",
            r#"[workspace]
members = [
    "crates/anyhow",
    "crates/tokio",
    "crates/modkit",
    "crates/local-module",
]
resolver = "3"
"#,
        );

        let result = (|| -> anyhow::Result<()> {
            env::set_current_dir(temp_dir.path())?;

            let current_dependencies = CargoTomlDependencies::from([(
                "local-module".to_owned(),
                CargoTomlDependency {
                    path: Some(
                        temp_dir
                            .path()
                            .join("crates/local-module")
                            .to_string_lossy()
                            .into_owned(),
                    ),
                    ..Default::default()
                },
            )]);

            generate_server_structure("generated", &current_dependencies)?;

            let generated_dir = generated_project_dir("generated")?;
            let generated_manifest = std::fs::read_to_string(generated_dir.join("Cargo.toml"))?;
            let cargo_toml: toml::Value = toml::from_str(&generated_manifest)?;
            let dependencies = cargo_toml
                .get("dependencies")
                .and_then(toml::Value::as_table)
                .expect("generated Cargo.toml should contain dependencies");
            let mut path_dependency_count = 0;

            for (name, dependency) in dependencies {
                let Some(path) = dependency
                    .as_table()
                    .and_then(|table| table.get("path"))
                    .and_then(toml::Value::as_str)
                else {
                    continue;
                };

                path_dependency_count += 1;
                let dependency_path = Path::new(path);
                assert!(
                    !dependency_path.is_absolute(),
                    "dependency {name} path should not be absolute: {path}"
                );
                assert!(
                    dependency_path.is_relative(),
                    "dependency {name} path should be relative: {path}"
                );
                assert!(
                    generated_dir.join(dependency_path).exists(),
                    "dependency {name} path should exist: {path}"
                );
            }

            assert!(path_dependency_count > 0);

            Ok(())
        })();

        result.expect("generate_server_structure should rewrite dependency paths");
    }
}
