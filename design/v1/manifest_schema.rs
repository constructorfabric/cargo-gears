use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default)]
    pub workspace: Workspace,
    pub apps: Apps,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub templates: Option<TemplateRegistry>,
}

pub type Apps = BTreeMap<String, Environments>;
pub type Environments = BTreeMap<String, Environment>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<PathBuf>,
    #[serde(default = "default_config_dir", rename = "config-dir")]
    pub config_dir: PathBuf,
    #[serde(default = "default_generated_dir", rename = "generated-dir")]
    pub generated_dir: PathBuf,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub config: PathBuf,
    pub test: TestPolicy,
    pub lint: LintPolicy,
    pub modules: Vec<ModuleRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<RunPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum ModuleRef {
    Local(LocalModuleRef),
    Remote(RemoteModuleRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalModuleRef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<VersionReq>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteModuleRef {
    pub name: String,
    pub version: VersionReq,
    pub package: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunPolicy {
    #[serde(default)]
    pub watch: WatchPolicy,
    #[serde(default)]
    pub fips: bool,
    #[serde(default)]
    pub otel: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WatchPolicy {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub paths: Vec<PathBuf>,
    #[serde(default)]
    pub ignore: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuildPolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<BuildProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub enum BuildProfile {
    Debug,
    Release,
    Custom(String),
}

impl From<String> for BuildProfile {
    fn from(value: String) -> Self {
        match value.as_str() {
            "debug" => Self::Debug,
            "release" => Self::Release,
            _ => Self::Custom(value),
        }
    }
}

impl From<BuildProfile> for String {
    fn from(profile: BuildProfile) -> Self {
        match profile {
            BuildProfile::Debug => "debug".to_string(),
            BuildProfile::Release => "release".to_string(),
            BuildProfile::Custom(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Dylint {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skip: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestPolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner: Option<TestRunner>,
    #[serde(default)]
    pub coverage: bool,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct TemplateRegistry {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub module: BTreeMap<String, TemplateSource>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<String, TemplateSource>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub agents: BTreeMap<String, TemplateSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestRunner {
    Cargo,
    Nextest,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ModuleFeatureSet {
    All(bool),
    Sets(Vec<FeatureSet>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FeatureSet {
    Disabled(bool),
    Features(Vec<String>),
}

const fn default_version() -> u32 {
    1
}

fn default_config_dir() -> PathBuf {
    PathBuf::from("config")
}

fn default_generated_dir() -> PathBuf {
    PathBuf::from(".cyberware")
}

const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest_example_toml() {
        let manifest: Manifest = toml::from_str(include_str!("manifest_example.toml")).unwrap();
        assert!(manifest.workspace.config_dir.ends_with("config"));
        assert_eq!(manifest.apps.len(), 1);

        let app = manifest.apps.get("app1").unwrap();
        assert_eq!(app.len(), 2);
        let dev = app.get("dev").unwrap();
        let _prod = app.get("prod").unwrap();
        assert!(dev.lint.clippy);
        assert_eq!(dev.test.runner.as_ref().unwrap(), &TestRunner::Nextest);
        // Add more assertions if required
    }

    #[test]
    fn rejects_environment_missing_policy_shape() {
        let err = toml::from_str::<Manifest>(
            r#"
[apps.app1.dev]
config = "app1-dev.yml"
modules = []
"#,
        )
        .unwrap_err();

        assert!(err.message().contains("missing field"));
    }
}
