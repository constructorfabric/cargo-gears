use crate::common;
use crate::gears_parser::{CargoTomlDependencies, CargoTomlDependency, ConfigModuleMetadata};
use anyhow::{Context, bail};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MANIFEST_FILE: &str = "Gears.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestSelection {
    pub manifest: PathBuf,
    pub app: Option<String>,
    pub env: Option<String>,
}

impl ManifestSelection {
    pub fn resolve(&self, workspace_root: &Path) -> anyhow::Result<ResolvedManifest> {
        let manifest_path = resolve_manifest_path(workspace_root, &self.manifest)?;
        let manifest = Manifest::load(&manifest_path)?;
        let (app, env) = resolve_app_env(&manifest, self.app.as_deref(), self.env.as_deref())?;
        manifest.resolve(workspace_root, &manifest_path, &app, &env, None)
    }
}

/// Resolve app and env from the manifest when not explicitly provided.
///
/// - If `app` is `None` and there is exactly one app, use it.
///   If there are multiple apps, bail with the available app names.
/// - If `env` is `None` and there is exactly one env, use it.
///   If there are multiple envs and one is called "dev", default to "dev".
///   Otherwise, bail with the available env names.
fn resolve_app_env(
    manifest: &Manifest,
    app: Option<&str>,
    env: Option<&str>,
) -> anyhow::Result<(String, String)> {
    let resolved_app = match app {
        Some(a) => a.to_owned(),
        None => match manifest.apps.len() {
            0 => bail!("no apps defined in manifest"),
            1 => manifest
                .apps
                .keys()
                .next()
                .context("single app should exist")
                .inspect(|x| {
                    println!("no app specified, defaulting to the only app in manifest: '{x}'");
                })?
                .clone(),
            _ => {
                let names: Vec<_> = manifest.apps.keys().collect();
                bail!(
                    "multiple apps in manifest, use --app to select one: {}",
                    names
                        .iter()
                        .map(|n| n.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        },
    };

    let envs = manifest
        .apps
        .get(&resolved_app)
        .with_context(|| format!("app '{resolved_app}' not found in manifest"))?;

    let resolved_env = match env {
        Some(e) => e.to_owned(),
        None if envs.contains_key("dev") => {
            println!("no env specified, defaulting to 'dev'");
            "dev".to_owned()
        }
        None => {
            let names: Vec<_> = envs.keys().collect();
            bail!(
                "no 'dev' environment for app '{resolved_app}', use --env to select one: {}",
                names
                    .iter()
                    .map(|n| n.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    };

    Ok((resolved_app, resolved_env))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestParams {
    pub path: Option<PathBuf>,
    pub manifest: PathBuf,
    pub command: ManifestCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestCommand {
    Validate { format: common::OutputFormat },
    Ls { format: common::OutputFormat },
}

impl ManifestParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_root = common::resolve_workspace_path(self.path.as_deref())?;
        let manifest_path = resolve_manifest_path(&workspace_root, &self.manifest)?;
        let manifest = Manifest::load(&manifest_path)?;

        match self.command {
            ManifestCommand::Validate { format } => {
                let report = manifest.validate(&workspace_root, &manifest_path);
                print_value(format, &report)
            }
            ManifestCommand::Ls { format } => {
                let entries = manifest.entries(&workspace_root, &manifest_path);
                match format {
                    common::OutputFormat::Table => {
                        for (i, e) in entries.iter().enumerate() {
                            if i > 0 {
                                println!();
                            }
                            println!("app: {}", e.app);
                            println!("env: {}", e.env);
                            if let Some(config) = &e.config {
                                println!("config: {}", config.display());
                            }
                            if let Some(name) = &e.name {
                                println!("name: {name}");
                            }
                        }
                        Ok(())
                    }
                    common::OutputFormat::Json => print_value(format, &entries),
                }
            }
        }
    }
}

fn print_value<T: Serialize>(format: common::OutputFormat, value: &T) -> anyhow::Result<()> {
    match format {
        common::OutputFormat::Json | common::OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
    }
    Ok(())
}

/// Top-level `Gears` manifest (`Gears.toml`).
///
/// Drives build, run, lint, and test workflows. Declares workspace-level
/// defaults and per-app/environment overrides.
/// Also defines templates
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Configuration for the
    #[serde(default)]
    pub workspace: Workspace,
    pub apps: Apps,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub templates: Option<TemplateRegistry>,
}

pub type Apps = BTreeMap<String, Environments>;
pub type Environments = BTreeMap<String, Environment>;

impl Manifest {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let manifest = fs::read_to_string(path)
            .with_context(|| format!("manifest not available at {}", path.display()))?;
        toml::from_str(&manifest)
            .with_context(|| format!("manifest not valid at {}", path.display()))
    }

    #[must_use]
    pub fn validate(&self, workspace_root: &Path, manifest_path: &Path) -> Vec<ValidationReport> {
        let mut entries = Vec::new();
        for (app, envs) in &self.apps {
            for env in envs.keys() {
                match self.resolve(workspace_root, manifest_path, app, env, None) {
                    Ok(r) => entries.push(ValidationReport {
                        error: None,
                        entry: ManifestEntry {
                            app: app.clone(),
                            env: env.clone(),
                            config: Some(r.config_path.clone()),
                            name: Some(r.generated_name.clone()),
                        },
                        info: Some(r),
                    }),
                    Err(err) => entries.push(ValidationReport {
                        error: Some(err.to_string()),
                        entry: ManifestEntry {
                            app: app.clone(),
                            env: env.clone(),
                            config: None,
                            name: None,
                        },
                        info: None,
                    }),
                }
            }
        }

        entries
    }

    #[must_use]
    pub fn entries(&self, workspace_root: &Path, manifest_path: &Path) -> Vec<ManifestEntry> {
        self.apps
            .iter()
            .flat_map(|(app, envs)| {
                envs.keys().map(|env| {
                    let (config, name) =
                        match self.resolve(workspace_root, manifest_path, app, env, None) {
                            Ok(resolved) => {
                                (Some(resolved.config_path), Some(resolved.generated_name))
                            }
                            Err(_) => (None, None),
                        };
                    ManifestEntry {
                        app: app.clone(),
                        env: env.clone(),
                        config,
                        name,
                    }
                })
            })
            .collect()
    }

    pub fn resolve(
        &self,
        workspace_root: &Path,
        manifest_path: &Path,
        app: &str,
        env: &str,
        config_override: Option<&Path>,
    ) -> anyhow::Result<ResolvedManifest> {
        let environment = self
            .apps
            .get(app)
            .with_context(|| format!("manifest app '{app}' does not exist"))?
            .get(env)
            .with_context(|| format!("manifest environment '{app}.{env}' does not exist"))?;
        let manifest_dir = manifest_path
            .parent()
            .context("manifest path has no parent")?;
        let workspace_base = self.workspace.root.as_ref().map_or_else(
            || workspace_root.to_path_buf(),
            |root| resolve_relative_to(manifest_dir, root),
        );
        let config_path = config_override.map_or_else(
            || {
                resolve_relative_to(
                    &workspace_base.join(&self.workspace.config_dir),
                    &environment.config,
                )
            },
            |config| resolve_relative_to(&workspace_base, config),
        );
        let generated_name = environment
            .build
            .as_ref()
            .and_then(|build| build.name.clone())
            .unwrap_or_else(|| format!("{app}-{env}"));
        let dependencies = resolve_dependencies(&workspace_base, &environment.gears)?;
        let generated_dir = resolve_relative_to(&workspace_base, &self.workspace.generated_dir);

        Ok(ResolvedManifest {
            app: app.to_owned(),
            env: env.to_owned(),
            manifest_path: manifest_path.to_path_buf(),
            workspace_root: workspace_base,
            generated_dir,
            config_path,
            generated_name,
            run: environment.run.clone().unwrap_or_default(),
            build: environment.build.clone().unwrap_or_default(),
            lint: environment.lint.clone(),
            test: environment.test.clone(),
            gears: environment.gears.clone(),
            dependencies,
        })
    }
}

pub fn resolve_manifest_path(workspace_root: &Path, manifest: &Path) -> anyhow::Result<PathBuf> {
    let path = resolve_relative_to(workspace_root, manifest);
    path.canonicalize()
        .with_context(|| format!("can't canonicalize manifest {}", path.display()))
}

fn resolve_relative_to(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn resolve_dependencies(
    workspace_root: &Path,
    gears: &[GearRef],
) -> anyhow::Result<CargoTomlDependencies> {
    let local_modules = gears
        .iter()
        .any(|module| matches!(module, GearRef::Local(_)))
        .then(|| crate::gears_parser::get_module_name_from_crate(Some(workspace_root)))
        .transpose()?
        .unwrap_or_default();
    let mut dependencies = CargoTomlDependencies::new();

    for module in gears {
        let (name, metadata) = match module {
            GearRef::Local(local) => {
                let discovered = local_modules.get(&local.name).with_context(|| {
                    format!("local module '{}' cannot be discovered", local.name)
                })?;
                let mut metadata = discovered.metadata.clone();
                if let Some(package) = &local.package {
                    metadata.package = Some(package.clone());
                }
                if let Some(version) = &local.version {
                    metadata.version = version_req_to_metadata(version);
                }
                if !local.features.is_empty() {
                    metadata.features.clone_from(&local.features);
                }
                if local.default_features.is_some() {
                    metadata.default_features = local.default_features;
                }
                (local.name.clone(), metadata)
            }
            GearRef::Remote(remote) => (
                remote.name.clone(),
                ConfigModuleMetadata {
                    package: Some(remote.package.clone()),
                    version: version_req_to_metadata(&remote.version),
                    features: remote.features.clone(),
                    default_features: remote.default_features,
                    ..Default::default()
                },
            ),
        };

        let package = metadata.package.clone().with_context(|| {
            format!("module '{name}' doesn't have package associated, please review")
        })?;
        let dependency_name = package.replace('-', "_");
        if dependencies.contains_key(&dependency_name) {
            bail!("multiple manifest modules resolve to package name '{dependency_name}'");
        }

        dependencies.insert(
            dependency_name,
            CargoTomlDependency {
                package: metadata.package,
                version: metadata.version,
                features: metadata.features.into_iter().collect(),
                default_features: metadata.default_features,
                path: metadata.path,
            },
        );
    }

    Ok(dependencies)
}

fn version_req_to_metadata(version: &VersionReq) -> Option<String> {
    let version = version.to_string();
    (version != "*").then_some(version)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedManifest {
    pub app: String,
    pub env: String,
    #[serde(skip_serializing)]
    pub manifest_path: PathBuf,
    pub workspace_root: PathBuf,
    pub generated_dir: PathBuf,
    pub config_path: PathBuf,
    pub generated_name: String,
    pub run: RunPolicy,
    pub build: BuildPolicy,
    pub lint: LintPolicy,
    pub test: TestPolicy,
    pub gears: Vec<GearRef>,
    pub dependencies: CargoTomlDependencies,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationReport {
    pub error: Option<String>,
    pub entry: ManifestEntry,
    pub info: Option<ResolvedManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestEntry {
    pub app: String,
    pub env: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Workspace-level defaults for paths and schema version.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    /// Schema version (currently always 1).
    #[serde(default = "default_version")]
    pub version: u32,
    /// Workspace root override (relative to manifest directory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<PathBuf>,
    /// Directory containing config YAML files.
    #[serde(default = "default_config_dir", rename = "config-dir")]
    pub config_dir: PathBuf,
    /// Directory for generated server projects.
    #[serde(default = "default_generated_dir", rename = "generated-dir")]
    pub generated_dir: PathBuf,
    /// Global environment inherited by all apps (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_env: Option<Environment>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            version: 1,
            root: None,
            config_dir: default_config_dir(),
            generated_dir: default_generated_dir(),
            global_env: None,
        }
    }
}

/// Per-app/environment configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    /// Config YAML path relative to config-dir.
    pub config: PathBuf,
    /// Test policy overrides.
    #[serde(default)]
    pub test: TestPolicy,
    /// Lint policy overrides.
    #[serde(default)]
    pub lint: LintPolicy,
    /// Gears to include in the generated server.
    #[serde(default)]
    pub gears: Vec<GearRef>,
    /// Runtime policy overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<RunPolicy>,
    /// Build policy overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum GearRef {
    Local(GearRefLocal),
    Remote(GearRefRemote),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GearRefLocal {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<VersionReq>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(
        default,
        rename = "default-features",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_features: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GearRefRemote {
    pub name: String,
    pub version: VersionReq,
    pub package: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(
        default,
        rename = "default-features",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_features: Option<bool>,
}

/// Runtime policy for watch, FIPS, and OpenTelemetry.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunPolicy {
    #[serde(default)]
    pub watch: WatchPolicy,
    #[serde(default)]
    pub fips: bool,
    #[serde(default)]
    pub otel: bool,
}

/// Watch-mode settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WatchPolicy {
    /// Enable file watching in run mode.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Paths to watch. When present, replaces the default watch set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<PathBuf>,
    /// Paths to exclude from the effective watch set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<PathBuf>,
}

impl Default for WatchPolicy {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            include: vec![],
            exclude: vec![],
        }
    }
}

/// Build policy controlling profile, name, and clean behavior.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildPolicy {
    /// Override generated project name (default: <app>-<env>).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Build profile (debug, release, or custom).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<BuildProfile>,
    /// Remove Cargo.lock before building.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(from = "String", into = "String")]
pub enum BuildProfile {
    Debug,
    Release,
    Custom(String),
}

impl From<String> for BuildProfile {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "debug" => Self::Debug,
            "release" => Self::Release,
            _ => Self::Custom(value),
        }
    }
}

impl From<BuildProfile> for String {
    fn from(profile: BuildProfile) -> Self {
        match profile {
            BuildProfile::Debug => "debug".to_owned(),
            BuildProfile::Release => "release".to_owned(),
            BuildProfile::Custom(value) => value,
        }
    }
}

/// Lint policy controlling clippy, fmt, dylint, and feature-set testing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LintPolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dylint: Option<Dylint>,
    #[serde(default = "default_true")]
    pub clippy: bool,
    #[serde(default = "default_true")]
    pub fmt: bool,
    #[serde(default = "default_true", rename = "feature-set-test")]
    pub feature_set_test: bool,
}

impl Default for LintPolicy {
    fn default() -> Self {
        Self {
            r#ref: None,
            dylint: None,
            clippy: true,
            fmt: true,
            feature_set_test: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
pub struct Dylint {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skip: Vec<String>,
}

/// Test policy controlling runner and feature-set testing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestPolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(default)]
    pub runner: TestRunner,
    #[serde(
        default,
        rename = "feature-set",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub feature_set: BTreeMap<String, ModuleFeatureSet>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "custom-command"
    )]
    pub custom_command: Option<String>,
}

/// Optional registry of template sources for generate commands.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields, default)]
pub struct TemplateRegistry {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module: Vec<TemplateDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config: Vec<TemplateDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<TemplateDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
pub struct TemplateDefinition {
    pub name: String,
    pub description: String,
    pub source: TemplateSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum TemplateSource {
    Git {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        revision: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subfolder: Option<String>,
    },
    Local {
        path: String,
    },
    Embedded,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum TestRunner {
    Cargo,
    #[default]
    Nextest,
}

pub type ModuleFeatureSet = Vec<FeatureSet>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum FeatureSet {
    DefaultFeatures,
    AllFeatures,
    NoDefaultFeatures,
    Features {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        features: Vec<String>,
    },
}

const fn default_version() -> u32 {
    1
}
fn default_config_dir() -> PathBuf {
    PathBuf::from("config")
}
fn default_generated_dir() -> PathBuf {
    PathBuf::from(".gears")
}
const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gears_parser::test_utils::TempDirExt;
    use tempfile::TempDir;

    #[test]
    fn parse_manifest_example_toml() {
        let manifest: Manifest =
            toml::from_str(include_str!("../../../design/v1/manifest_example.toml")).unwrap();
        assert!(manifest.workspace.config_dir.ends_with("config"));
        assert_eq!(manifest.apps.len(), 1);
    }

    #[test]
    fn watch_policy_uses_include_and_exclude_paths() {
        let policy: WatchPolicy = toml::from_str(
            r#"
enabled = true
include = ["crates/local-module", "Cargo.toml"]
exclude = ["target"]
"#,
        )
        .unwrap();

        assert_eq!(
            policy.include,
            vec![
                PathBuf::from("crates/local-module"),
                PathBuf::from("Cargo.toml")
            ]
        );
        assert_eq!(policy.exclude, vec![PathBuf::from("target")]);
    }

    #[test]
    fn render_resolves_remote_dependency_and_config_path() {
        let temp = TempDir::new().unwrap();
        temp.write(
            "Gears.toml",
            r#"
[workspace]
generated-dir = "custom-generated"

[apps.app.dev]
config = "app-dev.yml"
gears = [{ source = "remote", name = "module", package = "cf-module", version = "1.2" }]
"#,
        );
        temp.write("config/app-dev.yml", "server: {}\n");
        let manifest = Manifest::load(&temp.path().join(DEFAULT_MANIFEST_FILE)).unwrap();

        let resolved = manifest
            .resolve(
                temp.path(),
                &temp.path().join(DEFAULT_MANIFEST_FILE),
                "app",
                "dev",
                None,
            )
            .unwrap();

        assert_eq!(resolved.generated_name, "app-dev");
        assert_eq!(resolved.generated_dir, temp.path().join("custom-generated"));
        assert_eq!(resolved.config_path, temp.path().join("config/app-dev.yml"));
        assert!(resolved.dependencies.contains_key("cf_module"));
    }

    #[test]
    fn generated_dir_is_relative_to_manifest_workspace_root() {
        let temp = TempDir::new().unwrap();
        temp.write(
            "Gears.toml",
            r#"
[workspace]
root = "workspace"
generated-dir = "target/generated"

[apps.app.dev]
config = "app-dev.yml"
gears = [{ source = "remote", name = "module", package = "cf-module", version = "1.2" }]
"#,
        );
        let manifest = Manifest::load(&temp.path().join(DEFAULT_MANIFEST_FILE)).unwrap();

        let resolved = manifest
            .resolve(
                temp.path(),
                &temp.path().join(DEFAULT_MANIFEST_FILE),
                "app",
                "dev",
                None,
            )
            .unwrap();

        assert_eq!(resolved.workspace_root, temp.path().join("workspace"));
        assert_eq!(
            resolved.generated_dir,
            temp.path().join("workspace/target/generated")
        );
    }

    #[test]
    fn absolute_generated_dir_is_used_as_is() {
        let temp = TempDir::new().unwrap();
        let absolute_generated_dir = temp.path().join("absolute-generated");
        temp.write(
            "Gears.toml",
            &format!(
                r#"
[workspace]
root = "workspace"
generated-dir = "{}"

[apps.app.dev]
config = "app-dev.yml"
gears = [{{ source = "remote", name = "module", package = "cf-module", version = "1.2" }}]
"#,
                absolute_generated_dir.display()
            ),
        );
        let manifest = Manifest::load(&temp.path().join(DEFAULT_MANIFEST_FILE)).unwrap();

        let resolved = manifest
            .resolve(
                temp.path(),
                &temp.path().join(DEFAULT_MANIFEST_FILE),
                "app",
                "dev",
                None,
            )
            .unwrap();

        assert_eq!(resolved.workspace_root, temp.path().join("workspace"));
        assert_eq!(resolved.generated_dir, absolute_generated_dir);
    }

    #[test]
    fn manifest_args_resolve_manifest_relative_to_selected_workspace_path() {
        let temp = TempDir::new().unwrap();
        temp.write(
            "workspace/Gears.toml",
            r#"
[apps.app.dev]
config = "app-dev.yml"
gears = []
"#,
        );
        temp.write("workspace/config/app-dev.yml", "server: {}\n");

        let args = ManifestParams {
            path: Some(temp.path().join("workspace")),
            manifest: PathBuf::from(DEFAULT_MANIFEST_FILE),
            command: ManifestCommand::Ls {
                format: common::OutputFormat::Json,
            },
        };

        assert!(args.run().is_ok());
    }

    #[test]
    fn resolve_app_env_infers_single_app_and_env() {
        let manifest: Manifest = toml::from_str(
            r#"
[apps.myapp.dev]
config = "app-dev.yml"
"#,
        )
        .unwrap();
        let (app, env) = resolve_app_env(&manifest, None, None).unwrap();
        assert_eq!(app, "myapp");
        assert_eq!(env, "dev");
    }

    #[test]
    fn resolve_app_env_defaults_to_dev_with_multiple_envs() {
        let manifest: Manifest = toml::from_str(
            r#"
[apps.myapp.dev]
config = "dev.yml"
[apps.myapp.prod]
config = "prod.yml"
"#,
        )
        .unwrap();
        let (app, env) = resolve_app_env(&manifest, None, None).unwrap();
        assert_eq!(app, "myapp");
        assert_eq!(env, "dev");
    }

    #[test]
    fn resolve_app_env_fails_with_multiple_apps() {
        let manifest: Manifest = toml::from_str(
            r#"
[apps.app1.dev]
config = "a.yml"
[apps.app2.dev]
config = "b.yml"
"#,
        )
        .unwrap();
        let err = resolve_app_env(&manifest, None, None).unwrap_err();
        assert!(err.to_string().contains("multiple apps"));
    }

    #[test]
    fn resolve_app_env_fails_with_single_non_dev_env() {
        let manifest: Manifest = toml::from_str(
            r#"
[apps.myapp.prod]
config = "prod.yml"
"#,
        )
        .unwrap();
        let err = resolve_app_env(&manifest, None, None).unwrap_err();
        assert!(err.to_string().contains("no 'dev' environment"));
    }

    #[test]
    fn resolve_app_env_fails_with_multiple_non_dev_envs() {
        let manifest: Manifest = toml::from_str(
            r#"
[apps.myapp.staging]
config = "staging.yml"
[apps.myapp.prod]
config = "prod.yml"
"#,
        )
        .unwrap();
        let err = resolve_app_env(&manifest, None, None).unwrap_err();
        assert!(err.to_string().contains("no 'dev' environment"));
    }

    #[test]
    fn resolve_app_env_uses_explicit_values() {
        let manifest: Manifest = toml::from_str(
            r#"
[apps.app1.dev]
config = "a.yml"
[apps.app2.prod]
config = "b.yml"
"#,
        )
        .unwrap();
        let (app, env) = resolve_app_env(&manifest, Some("app2"), Some("prod")).unwrap();
        assert_eq!(app, "app2");
        assert_eq!(env, "prod");
    }
}
