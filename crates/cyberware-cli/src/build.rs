use crate::common::BuildRunArgs;
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[command(flatten)]
    build_run_args: BuildRunArgs,
}

impl BuildArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::build::BuildArgs {
            build_run_args: self.build_run_args.into(),
        }
        .run()
    }
}
