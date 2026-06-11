use crate::common::{ManifestTargetArgs, WorkspacePath};
use cargo_gears_core::manifest::TestRunner;
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
    /// Resolve manifest + CLI overrides into a fully-resolved `TestParams`.
    pub fn resolve(self) -> anyhow::Result<cargo_gears_core::test::TestParams> {
        let workspace_path =
            cargo_gears_core::common::resolve_workspace_path(self.workspace.path.as_deref())?;
        let resolved = self.manifest.into_selection().resolve(&workspace_path)?;

        let runner = self.runner.unwrap_or(resolved.test.runner);

        if self.runner.is_some() && resolved.test.custom_command.is_some() {
            eprintln!("WARN: custom command is specified in manifest, ignoring runner override");
        }

        Ok(cargo_gears_core::test::TestParams {
            workspace_root: resolved.workspace_root,
            config_path: resolved.config_path,
            runner,
            module: self.module,
            coverage: self.coverage,
            custom_command: resolved.test.custom_command,
            modules: resolved.modules,
            dependencies: resolved.dependencies,
            feature_set: resolved.test.feature_set,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TestArgs;
    use cargo_gears_core::manifest::TestRunner;
    use clap::Parser;
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
}
