use crate::common::{DbConnConfig, PathConfigArgs};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

impl ConfigArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cargo_gears_core::config::ConfigParams::from(self).run()
    }
}

impl From<ConfigArgs> for cargo_gears_core::config::ConfigParams {
    fn from(args: ConfigArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    Gear(ConfigGearsArgs),
    Db(Box<ConfigDbArgs>),
}

impl From<ConfigCommand> for cargo_gears_core::config::ConfigCommand {
    fn from(command: ConfigCommand) -> Self {
        match command {
            ConfigCommand::Gear(args) => Self::Mod(args.into()),
            ConfigCommand::Db(args) => Self::Db(Box::new((*args).into())),
        }
    }
}

#[derive(Args)]
pub struct ConfigDbArgs {
    #[command(subcommand)]
    command: ConfigDbCommand,
}

impl From<ConfigDbArgs> for cargo_gears_core::config::db::DbParams {
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

impl From<ConfigDbCommand> for cargo_gears_core::config::db::DbCommand {
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

impl From<ConfigDbAddArgs> for cargo_gears_core::config::db::AddArgs {
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

impl From<ConfigDbEditArgs> for cargo_gears_core::config::db::EditArgs {
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

impl From<ConfigDbRemoveArgs> for cargo_gears_core::config::db::RemoveArgs {
    fn from(args: ConfigDbRemoveArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            name: args.name,
        }
    }
}

#[derive(Args)]
pub struct ConfigGearsArgs {
    #[command(subcommand)]
    command: ConfigGearsCommand,
}

impl From<ConfigGearsArgs> for cargo_gears_core::config::gears::GearsParams {
    fn from(args: ConfigGearsArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

#[derive(Subcommand)]
pub enum ConfigGearsCommand {
    /// Add or update a gear in the gears section (upsert)
    Add(ConfigModuleAddArgs),
    /// Manage gear-level database config
    Db(Box<ConfigModuleDbArgs>),
    /// Remove a gear from the gears section
    Rm(ConfigModuleRemoveArgs),
}

impl From<ConfigGearsCommand> for cargo_gears_core::config::gears::GearsCommand {
    fn from(command: ConfigGearsCommand) -> Self {
        match command {
            ConfigGearsCommand::Add(args) => Self::Add(args.into()),
            ConfigGearsCommand::Db(args) => Self::Db(Box::new((*args).into())),
            ConfigGearsCommand::Rm(args) => Self::Rm(args.into()),
        }
    }
}

#[derive(Args)]
pub struct ConfigModuleAddArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Module name
    module: String,
    /// Dependency name to include in metadata.deps (repeatable)
    #[arg(long = "dep")]
    deps: Vec<String>,
}

impl From<ConfigModuleAddArgs> for cargo_gears_core::config::gears::add::AddParams {
    fn from(args: ConfigModuleAddArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            module: args.module,
            deps: args.deps,
        }
    }
}

#[derive(Args)]
pub struct ConfigModuleDbArgs {
    #[command(subcommand)]
    command: ConfigModuleDbCommand,
}

impl From<ConfigModuleDbArgs> for cargo_gears_core::config::gears::db::ModuleDbParams {
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

impl From<ConfigModuleDbCommand> for cargo_gears_core::config::gears::db::ModuleDbCommand {
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

impl From<ConfigModuleDbAddArgs> for cargo_gears_core::config::gears::db::AddArgs {
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

impl From<ConfigModuleDbEditArgs> for cargo_gears_core::config::gears::db::EditArgs {
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

impl From<ConfigModuleDbRemoveArgs> for cargo_gears_core::config::gears::db::RemoveArgs {
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

impl From<ConfigModuleRemoveArgs> for cargo_gears_core::config::gears::remove::RemoveParams {
    fn from(args: ConfigModuleRemoveArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            module: args.module,
        }
    }
}
