use crate::common::{ManifestTargetArgs, WorkspacePath};
use cargo_gears_core::manifest::TestRunner;
use clap::{ArgAction, Args};

#[derive(Args)]
pub struct TestArgs {
    #[command(flatten)]
    pub workspace: WorkspacePath,
    #[command(flatten)]
    pub manifest: ManifestTargetArgs,
    /// Test runner override.
    #[arg(long, value_enum)]
    pub runner: Option<TestRunner>,
    /// Limit tests to a module/package.
    #[arg(long)]
    pub module: Option<String>,
    /// Run test coverage.
    #[arg(long, action = ArgAction::SetTrue)]
    pub coverage: bool,
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
