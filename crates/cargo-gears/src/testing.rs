use crate::common::{ManifestTargetArgs, WorkspacePath};
use cargo_gears_core::gears_parser::CargoTomlDependencies;
use cargo_gears_core::manifest::{FeatureSet, ModuleFeatureSet, ModuleRef, TestRunner};
use cargo_gears_core::test::{FeatureSelection, TestPlan, TestRun};
use clap::{ArgAction, Args};

#[derive(Args)]
pub struct TestArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    #[command(flatten)]
    manifest: ManifestTargetArgs,
    /// Test runner override.
    #[arg(long, value_enum)]
    runner: Option<TestRunner>,
    /// Limit tests to a module/package.
    #[arg(long)]
    module: Option<String>,
    /// Run test coverage.
    #[arg(long, action = ArgAction::SetTrue)]
    coverage: bool,
}

impl TestArgs {
    /// Resolve manifest + CLI overrides into a fully-resolved `TestPlan`.
    pub fn resolve(self) -> anyhow::Result<TestPlan> {
        let workspace_path =
            cargo_gears_core::common::resolve_workspace_path(self.workspace.path.as_deref())?;
        let resolved = self.manifest.into_selection().resolve(&workspace_path)?;

        let runner = self.runner.unwrap_or(resolved.test.runner);

        if !self.coverage && self.runner.is_some() && resolved.test.custom_command.is_some() {
            eprintln!("WARN: custom command is specified in manifest, ignoring runner override");
        }

        let runs = resolve_runs(
            self.module.as_deref(),
            &resolved.test.feature_set,
            &resolved.modules,
            &resolved.dependencies,
        );

        Ok(TestPlan {
            workspace_root: resolved.workspace_root,
            config_path: resolved.config_path,
            runner,
            coverage: self.coverage,
            custom_command: resolved.test.custom_command,
            runs,
        })
    }
}

// --- Resolution logic (module name → package name, feature-set expansion) ---

fn resolve_runs(
    module: Option<&str>,
    feature_set: &std::collections::BTreeMap<String, ModuleFeatureSet>,
    modules: &[ModuleRef],
    dependencies: &CargoTomlDependencies,
) -> Vec<TestRun> {
    if feature_set.is_empty() {
        return vec![TestRun {
            package: module.map(|m| package_for_module(modules, dependencies, m)),
            features: FeatureSelection::Default,
        }];
    }

    if let Some(module) = module {
        let package = package_for_module(modules, dependencies, module);
        let actual_module =
            module_for_package(modules, &package).unwrap_or_else(|| module.to_owned());
        return match feature_set.get(&actual_module) {
            Some(set) => expand_feature_set(Some(package.as_str()), set),
            None => vec![TestRun {
                package: Some(package),
                features: FeatureSelection::Default,
            }],
        };
    }

    feature_set
        .iter()
        .flat_map(|(module, set)| {
            let package = package_for_module(modules, dependencies, module);
            expand_feature_set(Some(package.as_str()), set)
        })
        .collect()
}

fn expand_feature_set(package: Option<&str>, feature_set: &ModuleFeatureSet) -> Vec<TestRun> {
    if feature_set.is_empty() {
        return vec![TestRun {
            package: package.map(str::to_owned),
            features: FeatureSelection::Default,
        }];
    }

    feature_set
        .iter()
        .map(|set| TestRun {
            package: package.map(str::to_owned),
            features: feature_selection(set),
        })
        .collect()
}

fn feature_selection(set: &FeatureSet) -> FeatureSelection {
    match set {
        FeatureSet::DefaultFeatures => FeatureSelection::Default,
        FeatureSet::AllFeatures => FeatureSelection::AllFeatures,
        FeatureSet::NoDefaultFeatures => FeatureSelection::NoDefaultFeatures,
        FeatureSet::Features { features } => FeatureSelection::Features(features.clone()),
    }
}

fn package_for_module(
    modules: &[ModuleRef],
    dependencies: &CargoTomlDependencies,
    module: &str,
) -> String {
    if let Some(package) = package_from_dependencies(dependencies, module) {
        return package;
    }

    let normalized = module.replace('-', "_");
    if let Some(package) = package_from_dependencies(dependencies, &normalized) {
        return package;
    }

    modules
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

fn module_for_package(modules: &[ModuleRef], package: &str) -> Option<String> {
    modules.iter().find_map(|module_ref| match module_ref {
        ModuleRef::Local(local) if local.package.as_deref() == Some(package) => {
            Some(local.name.clone())
        }
        ModuleRef::Remote(remote) if remote.package == package => Some(remote.name.clone()),
        _ => None,
    })
}

fn package_from_dependencies(dependencies: &CargoTomlDependencies, module: &str) -> Option<String> {
    dependencies.get(module).map(|dependency| {
        dependency
            .package
            .clone()
            .unwrap_or_else(|| module.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_gears_core::gears_parser::CargoTomlDependencies;
    use cargo_gears_core::manifest::{FeatureSet, ModuleRef, RemoteModuleRef, TestRunner};
    use clap::Parser;
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::TempDir;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        test: TestArgs,
    }

    fn parse(temp: &TempDir, extra: &[&str]) -> TestArgs {
        let p = temp.path().to_str().expect("temp path should be UTF-8");
        let mut args = vec!["test", "-p", p, "--app", "app", "--env", "dev"];
        args.extend(extra);
        TestCli::try_parse_from(args).expect("should parse").test
    }

    fn write_workspace(temp: &TempDir, manifest: &str) {
        fs::write(temp.path().join("Gears.toml"), manifest).expect("write manifest");
        fs::create_dir_all(temp.path().join("config")).expect("create config dir");
        fs::write(temp.path().join("config/app-dev.yml"), "server: {}\n").expect("write config");
    }

    const MINIMAL: &str = "[apps.app.dev]\nconfig = \"app-dev.yml\"\nmodules = []\n";

    // --- CLI override tests ---

    #[test]
    fn defaults_to_manifest_runner() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            &format!("{MINIMAL}[apps.app.dev.test]\nrunner = \"cargo\"\n"),
        );

        let resolved = parse(&temp, &[]).resolve().expect("resolve");

        assert_eq!(resolved.runner, TestRunner::Cargo);
    }

    #[test]
    fn cli_runner_overrides_manifest() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            &format!("{MINIMAL}[apps.app.dev.test]\nrunner = \"cargo\"\n"),
        );

        let resolved = parse(&temp, &["--runner", "nextest"])
            .resolve()
            .expect("resolve");

        assert_eq!(resolved.runner, TestRunner::Nextest);
    }

    // --- Resolution logic tests ---

    fn sample_modules() -> Vec<ModuleRef> {
        vec![ModuleRef::Remote(RemoteModuleRef {
            name: "module-a".to_owned(),
            version: semver::VersionReq::STAR,
            package: "cf-module-a".to_owned(),
            registry: None,
            features: vec![],
        })]
    }

    #[test]
    fn default_plan_tests_workspace_once() {
        let runs = resolve_runs(
            None,
            &BTreeMap::new(),
            &sample_modules(),
            &CargoTomlDependencies::default(),
        );

        assert_eq!(
            runs,
            vec![TestRun {
                package: None,
                features: FeatureSelection::Default,
            }]
        );
    }

    #[test]
    fn feature_set_expands_module_matrix() {
        let feature_set = BTreeMap::from([(
            "module-a".to_owned(),
            vec![
                FeatureSet::Features {
                    features: vec!["sqlite".to_owned()],
                },
                FeatureSet::NoDefaultFeatures,
                FeatureSet::AllFeatures,
                FeatureSet::DefaultFeatures,
            ],
        )]);

        let runs = resolve_runs(
            None,
            &feature_set,
            &sample_modules(),
            &CargoTomlDependencies::default(),
        );

        assert_eq!(
            runs,
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
    fn empty_feature_set_falls_back_to_default_run() {
        let feature_set = BTreeMap::from([("module-a".to_owned(), vec![])]);

        let runs = resolve_runs(
            None,
            &feature_set,
            &sample_modules(),
            &CargoTomlDependencies::default(),
        );

        assert_eq!(
            runs,
            vec![TestRun {
                package: Some("cf-module-a".to_owned()),
                features: FeatureSelection::Default,
            }]
        );
    }

    #[test]
    fn cli_module_without_policy_tests_that_package() {
        let runs = resolve_runs(
            Some("module-a"),
            &BTreeMap::new(),
            &sample_modules(),
            &CargoTomlDependencies::default(),
        );

        assert_eq!(
            runs,
            vec![TestRun {
                package: Some("cf-module-a".to_owned()),
                features: FeatureSelection::Default,
            }]
        );
    }

    #[test]
    fn feature_set_lookup_works_with_package_name() {
        let feature_set = BTreeMap::from([(
            "module-a".to_owned(),
            vec![FeatureSet::Features {
                features: vec!["sqlite".to_owned()],
            }],
        )]);

        // Test with package name instead of module name
        let runs = resolve_runs(
            Some("cf-module-a"),
            &feature_set,
            &sample_modules(),
            &CargoTomlDependencies::default(),
        );

        assert_eq!(
            runs,
            vec![TestRun {
                package: Some("cf-module-a".to_owned()),
                features: FeatureSelection::Features(vec!["sqlite".to_owned()]),
            },]
        );
    }
}
