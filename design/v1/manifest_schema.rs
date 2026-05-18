use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default)]
    pub workspace: Workspace,
    pub env: Apps,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub templates: BTreeMap<String, toml::Value>,
}

pub type Apps = BTreeMap<String, App>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dev: Option<Environment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prod: Option<Environment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<TestSuites>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lint: Option<LintPolicy>,
}

pub type TestSuites = BTreeMap<String, TestPolicy>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_workspace_root")]
    pub root: String,
    #[serde(default = "default_config_dir", rename = "config-dir")]
    pub config_dir: String,
    #[serde(default = "default_generated_dir", rename = "generated-dir")]
    pub generated_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_env: Option<App>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            version: 1,
            root: default_workspace_root(),
            config_dir: default_config_dir(),
            generated_dir: default_generated_dir(),
            global_env: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub config: String,
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
    Registry(RegistryModuleRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalModuleRef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteModuleRef {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryModuleRef {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
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
    pub paths: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
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
#[serde(rename_all = "kebab-case")]
pub enum BuildProfile {
    Debug,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LintPolicy {
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
    pub runner: Option<TestRunner>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(default)]
    pub coverage: bool,
    #[serde(
        default,
        rename = "feature-set",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub feature_set: BTreeMap<String, ModuleFeatureSet>,
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

fn default_version() -> u32 {
    1
}

fn default_workspace_root() -> String {
    ".".to_owned()
}

fn default_config_dir() -> String {
    "config".to_owned()
}

fn default_generated_dir() -> String {
    ".cyberfabric".to_owned()
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
        assert_eq!(manifest.workspace.config_dir, "config");
        assert_eq!(manifest.env.len(), 1);

        let app = manifest.env.get("app1").unwrap();
        assert!(app.dev.is_some());
        assert!(app.prod.is_some());
        assert!(app.lint.is_some());
        assert!(app.test.as_ref().unwrap().contains_key("default"));
        // Add more assertions if required
    }

    #[test]
    fn rejects_environment_shape_under_lint() {
        let err = toml::from_str::<Manifest>(
            r#"
[env.app1.lint]
config = "app1-dev.yml"
modules = []
"#,
        )
        .unwrap_err();

        assert!(err.message().contains("unknown field `config`"));
    }
}
