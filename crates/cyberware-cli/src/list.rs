use crate::common::{OutputFormat, Registry};
use clap::{Args, Subcommand};
use std::path::PathBuf;

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
    /// List configuration files and their manifest links
    Configs(ConfigsArgs),
    /// List apps, environments, and build outputs
    Apps(AppsArgs),
}

#[derive(Args)]
pub struct ConfigsArgs {
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

#[derive(Args)]
pub struct AppsArgs {
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

#[derive(Args)]
pub struct ModulesArgs {
    /// Path to the module workspace root
    #[arg(short = 'p', long, value_parser = cyberware_cli_core::common::parse_and_chdir)]
    path: Option<PathBuf>,
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
    /// Path to the module workspace root
    #[arg(short = 'p', long, value_parser = cyberware_cli_core::common::parse_and_chdir)]
    path: Option<PathBuf>,
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
        cyberware_cli_core::list::ListArgs::from(self).run()
    }
}

impl From<ListArgs> for cyberware_cli_core::list::ListArgs {
    fn from(args: ListArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

impl From<ListCommand> for cyberware_cli_core::list::ListCommand {
    fn from(command: ListCommand) -> Self {
        match command {
            ListCommand::Modules(args) => Self::Modules(args.into()),
            ListCommand::LocalModules(args) => Self::LocalModules(args.into()),
            ListCommand::SystemModules(args) => Self::SystemModules(args.into()),
            ListCommand::Configs(args) => Self::Configs(args.into()),
            ListCommand::Apps(args) => Self::Apps(args.into()),
        }
    }
}

impl From<ModulesArgs> for cyberware_cli_core::list::ModulesArgs {
    fn from(args: ModulesArgs) -> Self {
        Self {
            path: args.path,
            verbose: args.verbose,
            registry: args.registry.into(),
            format: args.format.into(),
        }
    }
}

impl From<LocalModulesArgs> for cyberware_cli_core::list::LocalModulesArgs {
    fn from(args: LocalModulesArgs) -> Self {
        Self {
            path: args.path,
            verbose: args.verbose,
            format: args.format.into(),
        }
    }
}

impl From<SystemModulesArgs> for cyberware_cli_core::list::SystemModulesArgs {
    fn from(args: SystemModulesArgs) -> Self {
        Self {
            verbose: args.verbose,
            registry: args.registry.into(),
            format: args.format.into(),
        }
    }
}

impl From<ConfigsArgs> for cyberware_cli_core::list::ConfigsArgs {
    fn from(args: ConfigsArgs) -> Self {
        Self {
            format: args.format.into(),
        }
    }
}

impl From<AppsArgs> for cyberware_cli_core::list::AppsArgs {
    fn from(args: AppsArgs) -> Self {
        Self {
            format: args.format.into(),
        }
    }
}
