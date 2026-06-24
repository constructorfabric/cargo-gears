use crate::common::{OutputFormat, Registry, WorkspacePath};
use clap::{ArgAction, Args, Subcommand};

#[derive(Args)]
pub struct ListArgs {
    #[command(subcommand)]
    command: ListCommand,
}

#[derive(Subcommand)]
pub enum ListCommand {
    /// List gears available
    Gears(GearsArgs),
}

#[derive(Args)]
pub struct GearsArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    /// Show all information related to the gears (fetches registry metadata for system gears)
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Only list built-in system gears from the registry
    #[arg(long, action = ArgAction::SetTrue)]
    system: bool,
    /// Only list workspace-discovered gears
    #[arg(long, action = ArgAction::SetTrue)]
    local: bool,
    /// Registry to query for system-crate metadata
    #[arg(long, value_enum, default_value_t = Registry::CratesIo)]
    registry: Registry,
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
}

impl ListArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cargo_gears_core::list::ListParams::from(self).run()
    }
}

impl From<ListArgs> for cargo_gears_core::list::ListParams {
    fn from(args: ListArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

impl From<ListCommand> for cargo_gears_core::list::ListCommand {
    fn from(command: ListCommand) -> Self {
        match command {
            ListCommand::Gears(args) => Self::Gears(args.into()),
        }
    }
}

impl From<GearsArgs> for cargo_gears_core::list::GearsParams {
    fn from(args: GearsArgs) -> Self {
        let output = if args.system || args.local {
            match (args.system, args.local) {
                (true, false) => cargo_gears_core::list::GearsOutput::system(),
                (false, true) => cargo_gears_core::list::GearsOutput::local(),
                _ => cargo_gears_core::list::GearsOutput::all(),
            }
        } else {
            cargo_gears_core::list::GearsOutput::all()
        };

        Self {
            path: args.workspace.path,
            verbose: args.verbose,
            output,
            registry: args.registry,
            format: args.format,
        }
    }
}
