use crate::common::CONFIG_PATH_ENV_VAR;
use crate::manifest::{
    FeatureSet, ManifestSelection, ModuleFeatureSet, ModuleRef, ResolvedManifest, TestRunner,
};
use anyhow::{Context, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

mod cargo;
#[path = "llvm-cov.rs"]
mod llvm_cov;
mod nextest;

#[derive(Debug, Eq, PartialEq)]
pub struct TestParams {
    /// Path to the module workspace root.
    pub path: Option<PathBuf>,
    pub manifest: ManifestSelection,
    /// Test runner override. Defaults to the manifest policy, then nextest(built-in integrated).
    pub runner: Option<TestRunner>,
    /// Limit tests to a module/package.
    pub module: Option<String>,
    /// Run coverage with cargo llvm-cov.
    pub coverage: bool,
}

impl TestParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_path = crate::common::resolve_workspace_path(self.path.as_deref())?;
        let resolved = self.manifest.resolve(&workspace_path)?;
        let plan = TestPlan::new(&resolved, self.module.as_deref());

        let runner = self.runner.unwrap_or(resolved.test.runner);

        if self.coverage {
            return llvm_cov::run(&plan, runner);
        }

        if let Some(command) = &resolved.test.custom_command {
            if self.runner.is_some() {
                eprintln!(
                    "WARN: custom command is specified in manifest, ignoring runner override"
                );
            }
            return run_custom_command(command, &resolved.workspace_root, &resolved.config_path);
        }

        match runner {
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
    fn new(resolved: &ResolvedManifest, module: Option<&str>) -> Self {
        let runs = resolve_runs(resolved, module);
        Self {
            workspace_root: resolved.workspace_root.clone(),
            config_path: resolved.config_path.clone(),
            runs,
        }
    }
}

fn resolve_runs(resolved: &ResolvedManifest, module: Option<&str>) -> Vec<TestRun> {
    let policy = &resolved.test;
    if policy.feature_set.is_empty() {
        return vec![TestRun {
            package: module.map(|module| package_for_module(resolved, module)),
            features: FeatureSelection::Default,
        }];
    }

    if let Some(module) = module {
        let package = package_for_module(resolved, module);
        return match policy.feature_set.get(module) {
            Some(set) => expand_feature_set(Some(package.as_str()), set),
            None => vec![TestRun {
                package: Some(package),
                features: FeatureSelection::Default,
            }],
        };
    }

    policy
        .feature_set
        .iter()
        .flat_map(|(module, set)| {
            let package = package_for_module(resolved, module);
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

fn package_for_module(resolved: &ResolvedManifest, module: &str) -> String {
    if let Some(package) = package_from_dependencies(resolved, module) {
        return package;
    }

    let normalized = module.replace('-', "_");
    if let Some(package) = package_from_dependencies(resolved, &normalized) {
        return package;
    }

    resolved
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

fn package_from_dependencies(resolved: &ResolvedManifest, module: &str) -> Option<String> {
    resolved.dependencies.get(module).map(|dependency| {
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
    use crate::manifest::{BuildPolicy, LintPolicy, RunPolicy, TestPolicy};
    use crate::module_parser::CargoTomlDependencies;
    use std::collections::BTreeMap;

    fn resolved(test: TestPolicy) -> ResolvedManifest {
        ResolvedManifest {
            app: "app".to_owned(),
            env: "dev".to_owned(),
            manifest_path: PathBuf::from("/workspace/Gears.toml"),
            workspace_root: PathBuf::from("/workspace"),
            generated_dir: PathBuf::from("/workspace/.gears"),
            config_path: PathBuf::from("/workspace/config/app-dev.yml"),
            generated_name: "app-dev".to_owned(),
            run: RunPolicy::default(),
            build: BuildPolicy::default(),
            lint: LintPolicy::default(),
            test,
            modules: vec![ModuleRef::Remote(crate::manifest::RemoteModuleRef {
                name: "module-a".to_owned(),
                version: semver::VersionReq::STAR,
                package: "cf-module-a".to_owned(),
                registry: None,
            })],
            dependencies: CargoTomlDependencies::default(),
        }
    }

    #[test]
    fn default_plan_tests_workspace_once() {
        let plan = TestPlan::new(&resolved(TestPolicy::default()), None);

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
        let plan = TestPlan::new(&resolved(policy), None);

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
        let plan = TestPlan::new(&resolved(TestPolicy::default()), Some("module-a"));

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
