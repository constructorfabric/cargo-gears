use crate::common::BuildRunArgs;
use clap::Args;

#[derive(Args)]
pub struct RunArgs {
    /// Watch for changes
    #[arg(short = 'w', long)]
    watch: bool,
    #[command(flatten)]
    br_args: BuildRunArgs,
}

impl RunArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::run::RunArgs::from(self).run()
    }
}

impl From<RunArgs> for cyberware_cli_core::run::RunArgs {
    fn from(args: RunArgs) -> Self {
        Self {
            watch: args.watch,
            br_args: args.br_args.into(),
        }
    }
}
