use crate::common::BuildRunArgs;
use clap::{ArgAction, Args};

#[derive(Args)]
pub struct RunArgs {
    /// Watch for changes
    #[arg(short = 'w', long, action = ArgAction::SetTrue, conflicts_with = "no_watch")]
    watch: bool,
    /// Do not watch for changes
    #[arg(long = "no-watch", action = ArgAction::SetTrue, conflicts_with = "watch")]
    no_watch: bool,
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
            watch: match (args.watch, args.no_watch) {
                (true, false) => Some(true),
                (false, true) => Some(false),
                _ => None,
            },
            br_args: args.br_args.into(),
        }
    }
}
