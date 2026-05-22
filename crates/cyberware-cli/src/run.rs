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
        cyberware_cli_core::run::RunArgs {
            watch: self.watch,
            br_args: self.br_args.into(),
        }
        .run()
    }
}
