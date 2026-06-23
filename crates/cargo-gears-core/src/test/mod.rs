use crate::common::CONFIG_PATH_ENV_VAR;
use crate::manifest::TestRunner;
use anyhow::{Context, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

mod cargo;
#[path = "llvm-cov.rs"]
mod llvm_cov;
mod nextest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestPlan {
    /// Resolved workspace root path.
    pub workspace_root: PathBuf,
    /// Resolved config file path.
    pub config_path: PathBuf,
    /// Effective test runner (CLI override already applied).
    pub runner: TestRunner,
    /// Run coverage with cargo llvm-cov.
    pub coverage: bool,
    /// Custom test command from manifest (takes priority over runner).
    pub custom_command: Option<String>,
    /// Resolved test runs (package + feature selection per run).
    pub runs: Vec<TestRun>,
}

impl TestPlan {
    pub fn run(&self) -> anyhow::Result<()> {
        if self.coverage {
            return llvm_cov::run(self, self.runner);
        }

        if let Some(command) = &self.custom_command {
            return run_custom_command(command, &self.workspace_root, &self.config_path);
        }

        match self.runner {
            TestRunner::Cargo => cargo::run(self),
            TestRunner::Nextest => nextest::run(self),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestRun {
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
pub enum FeatureSelection {
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
