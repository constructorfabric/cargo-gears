use crate::common::{OutputFormat, Registry, WorkspacePath};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ListArgs {
    #[command(subcommand)]
    command: ListCommand,
}

#[derive(Subcommand)]
pub enum ListCommand {
    /// List all modules (system + workspace)
    Modules(ModulesArgs),
    /// List workspace-discovered modules
    LocalModules(LocalModulesArgs),
    /// List built-in system modules from the registry
    SystemModules(SystemModulesArgs),
}

#[derive(Args)]
pub struct ModulesArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    /// Show all information related to the modules (fetches registry metadata for system modules)
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Registry to query for system-crate metadata
    #[arg(long, value_enum, default_value_t = Registry::CratesIo)]
    registry: Registry,
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

#[derive(Args)]
pub struct LocalModulesArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    /// Show all information related to the module
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

#[derive(Args)]
pub struct SystemModulesArgs {
    /// Show all information related to the module (fetches registry metadata)
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Registry to query for system-crate metadata
    #[arg(long, value_enum, default_value_t = Registry::CratesIo)]
    registry: Registry,
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

impl ListArgs {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::list::ListParams::from(self).run()
    }
}

impl From<ListArgs> for gears_cli_core::list::ListParams {
    fn from(args: ListArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

impl From<ListCommand> for gears_cli_core::list::ListCommand {
    fn from(command: ListCommand) -> Self {
        match command {
            ListCommand::Modules(args) => Self::Modules(args.into()),
            ListCommand::LocalModules(args) => Self::LocalModules(args.into()),
            ListCommand::SystemModules(args) => Self::SystemModules(args.into()),
        }
    }
}

impl From<ModulesArgs> for gears_cli_core::list::ModulesParams {
    fn from(args: ModulesArgs) -> Self {
        Self {
            path: args.workspace.path,
            verbose: args.verbose,
            registry: args.registry,
            format: args.format,
        }
    }
}

impl From<LocalModulesArgs> for gears_cli_core::list::LocalModulesParams {
    fn from(args: LocalModulesArgs) -> Self {
        Self {
            path: args.workspace.path,
            verbose: args.verbose,
            format: args.format,
        }
    }
}

impl From<SystemModulesArgs> for gears_cli_core::list::SystemModulesParams {
    fn from(args: SystemModulesArgs) -> Self {
        Self {
            verbose: args.verbose,
            registry: args.registry,
            format: args.format,
        }
    }
}
