use crate::common::BuildRunArgs;
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[command(flatten)]
    build_run_args: BuildRunArgs,
}

impl BuildArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cargo_gears_core::build::BuildParams::from(self).run()
    }
}

impl From<BuildArgs> for cargo_gears_core::build::BuildParams {
    fn from(args: BuildArgs) -> Self {
        Self {
            build_run_args: args.build_run_args.into(),
        }
    }
}
