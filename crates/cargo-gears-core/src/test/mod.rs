use crate::common::CONFIG_PATH_ENV_VAR;
use crate::gears_parser::CargoTomlDependencies;
use crate::manifest::{FeatureSet, ModuleFeatureSet, ModuleRef, TestRunner};
use anyhow::{Context, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

mod cargo;
#[path = "llvm-cov.rs"]
mod llvm_cov;
mod nextest;

#[derive(Debug, Eq, PartialEq)]
pub struct TestParams {
    /// Resolved workspace root path.
    pub workspace_root: PathBuf,
    /// Resolved config file path.
    pub config_path: PathBuf,
    /// Effective test runner (CLI override already applied).
    pub runner: TestRunner,
    /// Limit tests to a module/package.
    pub module: Option<String>,
    /// Run coverage with cargo llvm-cov.
    pub coverage: bool,
    /// Custom test command from manifest (takes priority over runner).
    pub custom_command: Option<String>,
    /// Module references for package resolution.
    pub modules: Vec<ModuleRef>,
    /// Workspace dependencies for package resolution.
    pub dependencies: CargoTomlDependencies,
    /// Feature-set matrix from test policy.
    pub feature_set: std::collections::BTreeMap<String, ModuleFeatureSet>,
}

impl TestParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let plan = TestPlan::new(self);

        if self.coverage {
            return llvm_cov::run(&plan, self.runner);
        }

        if let Some(command) = &self.custom_command {
            return run_custom_command(command, &self.workspace_root, &self.config_path);
        }

        match self.runner {
            TestRunner::Cargo => cargo::run(&plan),
            TestRunner::Nextest => nextest::run(&plan),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TestPlan {
    pub workspace_root: PathBuf,
    pub config_path: PathBuf,
    pub runs: Vec<TestRun>,
}

impl TestPlan {
    fn new(params: &TestParams) -> Self {
        let runs = resolve_runs(params);
        Self {
            workspace_root: params.workspace_root.clone(),
            config_path: params.config_path.clone(),
            runs,
        }
    }
}

fn resolve_runs(params: &TestParams) -> Vec<TestRun> {
    if params.feature_set.is_empty() {
        return vec![TestRun {
            package: params
                .module
                .as_deref()
                .map(|module| package_for_module(params, module)),
            features: FeatureSelection::Default,
        }];
    }

    if let Some(module) = params.module.as_deref() {
        let package = package_for_module(params, module);
        return match params.feature_set.get(module) {
            Some(set) => expand_feature_set(Some(package.as_str()), set),
            None => vec![TestRun {
                package: Some(package),
                features: FeatureSelection::Default,
            }],
        };
    }

    params
        .feature_set
        .iter()
        .flat_map(|(module, set)| {
            let package = package_for_module(params, module);
            expand_feature_set(Some(package.as_str()), set)
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TestRun {
    pub package: Option<String>,
    pub features: FeatureSelection,
}

impl TestRun {
    pub fn append_cargo_args(&self, args: &mut Vec<String>) {
        if let Some(package) = &self.package {
            args.push("-p".to_owned());
            args.push(package.clone());
        } else {
            args.push("--workspace".to_owned());
        }
        self.features.append_cargo_args(args);
    }

    pub fn append_cargo_metadata_args(&self, args: &mut Vec<String>) {
        match &self.features {
            FeatureSelection::Default => {}
            FeatureSelection::AllFeatures => args.push("--all-features".to_owned()),
            FeatureSelection::NoDefaultFeatures => args.push("--no-default-features".to_owned()),
            FeatureSelection::Features(features) => {
                args.push("--no-default-features".to_owned());
                if !features.is_empty() {
                    args.push("--features".to_owned());
                    args.push(self.metadata_features(features));
                }
            }
        }
    }

    fn metadata_features(&self, features: &[String]) -> String {
        self.package.as_ref().map_or_else(
            || features.join(","),
            |package| {
                features
                    .iter()
                    .map(|feature| format!("{package}/{feature}"))
                    .collect::<Vec<_>>()
                    .join(",")
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FeatureSelection {
    Default,
    AllFeatures,
    NoDefaultFeatures,
    Features(Vec<String>),
}

impl FeatureSelection {
    pub fn append_cargo_args(&self, args: &mut Vec<String>) {
        match self {
            Self::Default => {}
            Self::AllFeatures => args.push("--all-features".to_owned()),
            Self::NoDefaultFeatures => args.push("--no-default-features".to_owned()),
            Self::Features(features) => {
                args.push("--no-default-features".to_owned());
                if !features.is_empty() {
                    args.push("--features".to_owned());
                    args.push(features.join(","));
                }
            }
        }
    }
}

fn expand_feature_set(package: Option<&str>, feature_set: &ModuleFeatureSet) -> Vec<TestRun> {
    feature_set
        .iter()
        .map(|set| TestRun {
            package: package.map(str::to_owned),
            features: set.into(),
        })
        .collect()
}

impl From<&FeatureSet> for FeatureSelection {
    fn from(set: &FeatureSet) -> Self {
        match set {
            FeatureSet::DefaultFeatures => Self::Default,
            FeatureSet::AllFeatures => Self::AllFeatures,
            FeatureSet::NoDefaultFeatures => Self::NoDefaultFeatures,
            FeatureSet::Features { features } => Self::Features(features.clone()),
        }
    }
}

fn package_for_module(params: &TestParams, module: &str) -> String {
    if let Some(package) = package_from_dependencies(&params.dependencies, module) {
        return package;
    }

    let normalized = module.replace('-', "_");
    if let Some(package) = package_from_dependencies(&params.dependencies, &normalized) {
        return package;
    }

    params
        .modules
        .iter()
        .find_map(|module_ref| match module_ref {
            ModuleRef::Local(local)
                if local.name == module || local.package.as_deref() == Some(module) =>
            {
                Some(local.package.clone().unwrap_or_else(|| local.name.clone()))
            }
            ModuleRef::Remote(remote) if remote.name == module || remote.package == module => {
                Some(remote.package.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| module.to_owned())
}

fn package_from_dependencies(dependencies: &CargoTomlDependencies, module: &str) -> Option<String> {
    dependencies.get(module).map(|dependency| {
        dependency
            .package
            .clone()
            .unwrap_or_else(|| module.to_owned())
    })
}

fn run_custom_command(
    command: &str,
    workspace_root: &Path,
    config_path: &Path,
) -> anyhow::Result<()> {
    let parts = shell_words::split(command)
        .with_context(|| format!("failed to parse test custom-command `{command}`"))?;
    let (program, parts) = parts
        .split_first()
        .context("test custom-command must not be empty")?;

    let status = Command::new(program)
        .args(parts)
        .current_dir(workspace_root)
        .env(CONFIG_PATH_ENV_VAR, config_path)
        .status()
        .with_context(|| format!("failed to run test custom-command `{command}`"))?;

    if !status.success() {
        bail!("test custom-command `{command}` exited with {status}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::TestPolicy;
    use std::collections::BTreeMap;

    fn test_params(test: TestPolicy, module: Option<&str>) -> TestParams {
        TestParams {
            workspace_root: PathBuf::from("/workspace"),
            config_path: PathBuf::from("/workspace/config/app-dev.yml"),
            runner: test.runner,
            module: module.map(str::to_owned),
            coverage: false,
            custom_command: test.custom_command.clone(),
            modules: vec![ModuleRef::Remote(crate::manifest::RemoteModuleRef {
                name: "module-a".to_owned(),
                version: semver::VersionReq::STAR,
                package: "cf-module-a".to_owned(),
                registry: None,
            })],
            dependencies: CargoTomlDependencies::default(),
            feature_set: test.feature_set,
        }
    }

    #[test]
    fn default_plan_tests_workspace_once() {
        let params = test_params(TestPolicy::default(), None);
        let plan = TestPlan::new(&params);

        assert_eq!(
            plan.runs,
            vec![TestRun {
                package: None,
                features: FeatureSelection::Default,
            }]
        );
    }

    #[test]
    fn feature_set_expands_module_matrix() {
        let policy = TestPolicy {
            feature_set: BTreeMap::from([(
                "module-a".to_owned(),
                vec![
                    FeatureSet::Features {
                        features: vec!["sqlite".to_owned()],
                    },
                    FeatureSet::NoDefaultFeatures,
                    FeatureSet::AllFeatures,
                    FeatureSet::DefaultFeatures,
                ],
            )]),
            ..Default::default()
        };
        let params = test_params(policy, None);
        let plan = TestPlan::new(&params);

        assert_eq!(
            plan.runs,
            vec![
                TestRun {
                    package: Some("cf-module-a".to_owned()),
                    features: FeatureSelection::Features(vec!["sqlite".to_owned()]),
                },
                TestRun {
                    package: Some("cf-module-a".to_owned()),
                    features: FeatureSelection::NoDefaultFeatures,
                },
                TestRun {
                    package: Some("cf-module-a".to_owned()),
                    features: FeatureSelection::AllFeatures,
                },
                TestRun {
                    package: Some("cf-module-a".to_owned()),
                    features: FeatureSelection::Default,
                },
            ]
        );
    }

    #[test]
    fn cli_module_without_policy_tests_that_package() {
        let params = test_params(TestPolicy::default(), Some("module-a"));
        let plan = TestPlan::new(&params);

        assert_eq!(
            plan.runs,
            vec![TestRun {
                package: Some("cf-module-a".to_owned()),
                features: FeatureSelection::Default,
            }]
        );
    }

    #[test]
    fn metadata_feature_args_qualify_package_features() {
        let run = TestRun {
            package: Some("cf-module-a".to_owned()),
            features: FeatureSelection::Features(vec!["sqlite".to_owned(), "otel".to_owned()]),
        };
        let mut args = Vec::new();

        run.append_cargo_metadata_args(&mut args);

        assert_eq!(
            args,
            vec![
                "--no-default-features",
                "--features",
                "cf-module-a/sqlite,cf-module-a/otel"
            ]
        );
    }
}
