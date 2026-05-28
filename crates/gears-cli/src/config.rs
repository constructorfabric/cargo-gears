use crate::common::{DbConnConfig, PathConfigArgs, Registry};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

impl ConfigArgs {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::config::ConfigArgs::from(self).run()
    }
}

impl From<ConfigArgs> for gears_cli_core::config::ConfigArgs {
    fn from(args: ConfigArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    Mod(ConfigModulesArgs),
    Db(Box<ConfigDbArgs>),
}

impl From<ConfigCommand> for gears_cli_core::config::ConfigCommand {
    fn from(command: ConfigCommand) -> Self {
        match command {
            ConfigCommand::Mod(args) => Self::Mod(args.into()),
            ConfigCommand::Db(args) => Self::Db(Box::new((*args).into())),
        }
    }
}

#[derive(Args)]
pub struct ConfigDbArgs {
    #[command(subcommand)]
    command: ConfigDbCommand,
}

impl From<ConfigDbArgs> for gears_cli_core::config::db::DbArgs {
    fn from(args: ConfigDbArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

#[derive(Subcommand)]
enum ConfigDbCommand {
    /// Add or update (upsert) a global database server config under `database.servers`
    Add(ConfigDbAddArgs),
    /// Edit a global database server config under `database.servers`
    Edit(ConfigDbEditArgs),
    /// Remove a global database server config from `database.servers`
    Rm(ConfigDbRemoveArgs),
}

impl From<ConfigDbCommand> for gears_cli_core::config::db::DbCommand {
    fn from(command: ConfigDbCommand) -> Self {
        match command {
            ConfigDbCommand::Add(args) => Self::Add(args.into()),
            ConfigDbCommand::Edit(args) => Self::Edit(args.into()),
            ConfigDbCommand::Rm(args) => Self::Rm(args.into()),
        }
    }
}

#[derive(Args)]
struct ConfigDbAddArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Server name under `database.servers.<name>`
    name: String,
    #[command(flatten)]
    conn: DbConnConfig,
}

impl From<ConfigDbAddArgs> for gears_cli_core::config::db::AddArgs {
    fn from(args: ConfigDbAddArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            name: args.name,
            conn: args.conn.into(),
        }
    }
}

#[derive(Args)]
struct ConfigDbEditArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Server name under `database.servers.<name>`
    name: String,
    #[command(flatten)]
    conn: DbConnConfig,
}

impl From<ConfigDbEditArgs> for gears_cli_core::config::db::EditArgs {
    fn from(args: ConfigDbEditArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            name: args.name,
            conn: args.conn.into(),
        }
    }
}

#[derive(Args)]
struct ConfigDbRemoveArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Server name under `database.servers.<name>`
    name: String,
}

impl From<ConfigDbRemoveArgs> for gears_cli_core::config::db::RemoveArgs {
    fn from(args: ConfigDbRemoveArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            name: args.name,
        }
    }
}

#[derive(Args)]
pub struct ConfigModulesArgs {
    #[command(subcommand)]
    command: ConfigModulesCommand,
}

impl From<ConfigModulesArgs> for gears_cli_core::config::modules::ModulesArgs {
    fn from(args: ConfigModulesArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

#[derive(Subcommand)]
pub enum ConfigModulesCommand {
    /// List available system crates
    List(ConfigModuleListArgs),
    /// Add or update a module in the modules section (upsert)
    Add(ConfigModuleAddArgs),
    /// Manage module-level database config
    Db(Box<ConfigModuleDbArgs>),
    /// Remove a module from the modules section
    Rm(ConfigModuleRemoveArgs),
}

impl From<ConfigModulesCommand> for gears_cli_core::config::modules::ModulesCommand {
    fn from(command: ConfigModulesCommand) -> Self {
        match command {
            ConfigModulesCommand::List(args) => Self::List(args.into()),
            ConfigModulesCommand::Add(args) => Self::Add(args.into()),
            ConfigModulesCommand::Db(args) => Self::Db(Box::new((*args).into())),
            ConfigModulesCommand::Rm(args) => Self::Rm(args.into()),
        }
    }
}

#[derive(Args)]
pub struct ConfigModuleListArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Show system crates also. If verbose is enabled,
    /// fetches registry metadata for system crates. (makes requests to the registry)
    #[arg(short = 's', long)]
    system: bool,
    /// Show all information related to the module.
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Registry to query for system-crate metadata. Only consulted when both
    /// `--system` and `--verbose` are enabled; `--verbose` alone does not query
    /// any registry. Defaults to `crates.io`.
    #[arg(long, value_enum, default_value_t = Registry::CratesIo)]
    registry: Registry,
}

impl From<ConfigModuleListArgs> for gears_cli_core::config::modules::list::ListArgs {
    fn from(args: ConfigModuleListArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            system: args.system,
            verbose: args.verbose,
            registry: args.registry,
        }
    }
}

#[derive(Args)]
pub struct ConfigModuleAddArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Module name
    module: String,
    /// Module package name for metadata
    #[arg(long)]
    package: Option<String>,
    /// Module package version for metadata
    #[arg(long = "module-version")]
    module_version: Option<String>,
    /// Whether Cargo default features should be enabled
    #[arg(long)]
    default_features: Option<bool>,
    /// Feature to include in metadata (repeatable)
    #[arg(short = 'F', long = "feature", value_delimiter = ',')]
    features: Vec<String>,
    /// Dependency name to include in metadata.deps (repeatable)
    #[arg(long = "dep")]
    deps: Vec<String>,
}

impl From<ConfigModuleAddArgs> for gears_cli_core::config::modules::add::AddArgs {
    fn from(args: ConfigModuleAddArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            module: args.module,
            package: args.package,
            module_version: args.module_version,
            default_features: args.default_features,
            features: args.features,
            deps: args.deps,
        }
    }
}

#[derive(Args)]
pub struct ConfigModuleDbArgs {
    #[command(subcommand)]
    command: ConfigModuleDbCommand,
}

impl From<ConfigModuleDbArgs> for gears_cli_core::config::modules::db::ModuleDbArgs {
    fn from(args: ConfigModuleDbArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

#[derive(Subcommand)]
enum ConfigModuleDbCommand {
    /// Add or update (upsert) a module-level database configuration
    Add(ConfigModuleDbAddArgs),
    /// Edit a module-level database configuration
    Edit(ConfigModuleDbEditArgs),
    /// Remove a module-level database configuration
    Rm(ConfigModuleDbRemoveArgs),
}

impl From<ConfigModuleDbCommand> for gears_cli_core::config::modules::db::ModuleDbCommand {
    fn from(command: ConfigModuleDbCommand) -> Self {
        match command {
            ConfigModuleDbCommand::Add(args) => Self::Add(args.into()),
            ConfigModuleDbCommand::Edit(args) => Self::Edit(args.into()),
            ConfigModuleDbCommand::Rm(args) => Self::Rm(args.into()),
        }
    }
}

#[derive(Args)]
struct ConfigModuleDbAddArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Module name under `modules.<module>`
    module: String,
    #[command(flatten)]
    conn: DbConnConfig,
}

impl From<ConfigModuleDbAddArgs> for gears_cli_core::config::modules::db::AddArgs {
    fn from(args: ConfigModuleDbAddArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            module: args.module,
            conn: args.conn.into(),
        }
    }
}

#[derive(Args)]
struct ConfigModuleDbEditArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Module name under `modules.<module>`
    module: String,
    #[command(flatten)]
    conn: DbConnConfig,
}

impl From<ConfigModuleDbEditArgs> for gears_cli_core::config::modules::db::EditArgs {
    fn from(args: ConfigModuleDbEditArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            module: args.module,
            conn: args.conn.into(),
        }
    }
}

#[derive(Args)]
struct ConfigModuleDbRemoveArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Module name under `modules.<module>`
    module: String,
}

impl From<ConfigModuleDbRemoveArgs> for gears_cli_core::config::modules::db::RemoveArgs {
    fn from(args: ConfigModuleDbRemoveArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            module: args.module,
        }
    }
}

#[derive(Args)]
pub struct ConfigModuleRemoveArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Module name
    module: String,
}

impl From<ConfigModuleRemoveArgs> for gears_cli_core::config::modules::remove::RemoveArgs {
    fn from(args: ConfigModuleRemoveArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            module: args.module,
        }
    }
}
