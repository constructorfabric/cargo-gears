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
    pub watch: bool,
    #[serde(default)]
    pub fips: bool,
    #[serde(default)]
    pub otel: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuildPolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<BuildOutput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<BuildProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerBuild>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildOutput {
    Binary,
    Docker,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildProfile {
    Debug,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DockerBuild {
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default = "default_dockerfile")]
    pub dockerfile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LintPolicy {
    #[serde(
        default,
        rename = "skip-dylint",
        skip_serializing_if = "Option::is_none"
    )]
    pub skip_dylint: Option<SkipDylint>,
    #[serde(default = "default_true")]
    pub clippy: bool,
    #[serde(default = "default_true")]
    pub fmt: bool,
    #[serde(default = "default_true", rename = "feature-set-test")]
    pub feature_set_test: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SkipDylint {
    All(bool),
    Rules(Vec<String>),
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

fn default_workspace_root() -> String {
    ".".to_owned()
}

fn default_config_dir() -> String {
    "config".to_owned()
}

fn default_generated_dir() -> String {
    ".cyberfabric".to_owned()
}

fn default_dockerfile() -> String {
    "Dockerfile".to_owned()
}

const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest_example() {
        let manifest: Manifest = toml::from_str(
            r#"
[workspace]
root = "."
config-dir = "config"
generated-dir = ".cyberfabric"

[env.app1.dev]
config = "app1-dev.yml"
modules = [
    { name = "module1", source = "local", version = "1.2.0", package = "crate1" },
    { name = "module2", source = "remote", version = "0.1.0", package = "crate2" }
]
run = { watch = true, fips = false, otel = true }
build = { name = "app1", outputs = ["binary", "docker"], image = "registry.example.com/app1", profile = "debug" }

[env.app1.lint]
skip-dylint = ["rule-name"]
clippy = true
fmt = true
feature-set-test = true

[env.app1.test.default]
runner = "nextest"
config = "app1-test.yml"
coverage = true
feature-set = {
    "module1" = [["unit", "integration"], ["sqlite"], ["postgres"], ["fips"], false],
    "module2" = true,
}

[env.app1.prod]
config = "app1-prod.yml"
modules = [
    { name = "module1", source = "local", version = "1.2.0", package = "crate1" },
    { name = "module2", source = "remote", version = "0.1.0", package = "crate2" }
]
run = { watch = false, fips = true, otel = true }

[env.app1.prod.build]
name = "app1"
profile = "release"

[env.app1.prod.build.docker]
image = "registry.example.com/app1"
"#,
        )
        .unwrap();

        assert_eq!(manifest.workspace.config_dir, "config");
        let app = manifest.env.get("app1").unwrap();
        assert!(app.dev.is_some());
        assert!(app.prod.is_some());
        assert!(app.lint.is_some());
        assert!(app.test.as_ref().unwrap().contains_key("default"));
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
