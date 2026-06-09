use crate::common::{OutputFormat, Registry, WorkspacePath};
use clap::{ArgAction, Args, Subcommand};

#[derive(Args)]
pub struct ListArgs {
    #[command(subcommand)]
    command: ListCommand,
}

#[derive(Subcommand)]
pub enum ListCommand {
    /// List modules
    Modules(ModulesArgs),
}

#[derive(Args)]
pub struct ModulesArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    /// Show all information related to the modules (fetches registry metadata for system modules)
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Only list built-in system modules from the registry
    #[arg(long, action = ArgAction::SetTrue)]
    system: bool,
    /// Only list workspace-discovered modules
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
            ListCommand::Modules(args) => Self::Modules(args.into()),
        }
    }
}

impl From<ModulesArgs> for cargo_gears_core::list::ModulesParams {
    fn from(args: ModulesArgs) -> Self {
        let output = if args.system || args.local {
            match (args.system, args.local) {
                (true, false) => cargo_gears_core::list::ModulesOutput::system(),
                (false, true) => cargo_gears_core::list::ModulesOutput::local(),
                _ => cargo_gears_core::list::ModulesOutput::all(),
            }
        } else {
            cargo_gears_core::list::ModulesOutput::all()
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
