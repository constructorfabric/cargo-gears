use crate::common::BuildRunArgs;
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[command(flatten)]
    build_run_args: BuildRunArgs,
}

impl BuildArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::build::BuildArgs::from(self).run()
    }
}

impl From<BuildArgs> for cyberware_cli_core::build::BuildArgs {
    fn from(args: BuildArgs) -> Self {
        Self {
            build_run_args: args.build_run_args.into(),
        }
    }
}
