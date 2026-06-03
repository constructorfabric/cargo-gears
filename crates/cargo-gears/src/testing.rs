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
    pub fn run(self) -> anyhow::Result<()> {
        cargo_gears_core::test::TestParams::from(self).run()
    }
}

impl From<TestArgs> for cargo_gears_core::test::TestParams {
    fn from(args: TestArgs) -> Self {
        Self {
            path: args.workspace.path,
            manifest: args.manifest.into_selection(),
            runner: args.runner,
            module: args.module,
            coverage: args.coverage,
        }
    }
}
