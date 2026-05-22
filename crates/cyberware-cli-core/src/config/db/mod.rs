use super::{ensure_conn_payload, load_config, save_config, validate_name};
use crate::app_config::DbConnConfig;
use crate::common::PathConfigArgs;
use anyhow::{Context, bail};

pub struct DbArgs {
    pub command: DbCommand,
}

pub enum DbCommand {
    /// Add or update (upsert) a global database server config under `database.servers`
    Add(AddArgs),
    /// Edit a global database server config under `database.servers`
    Edit(EditArgs),
    /// Remove a global database server config from `database.servers`
    Rm(RemoveArgs),
}

impl DbArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            DbCommand::Add(args) => args.run(),
            DbCommand::Edit(args) => args.run(),
            DbCommand::Rm(args) => args.run(),
        }
    }
}

pub struct AddArgs {
    pub path_config: PathConfigArgs,
    /// Server name under `database.servers.<name>`
    pub name: String,
    pub conn: DbConnConfig,
}

impl AddArgs {
    fn run(&self) -> anyhow::Result<()> {
        let config_path = self.path_config.resolve_config()?;
        validate_name(&self.name, "server")?;
        ensure_conn_payload(&self.conn)?;

        let mut config = load_config(&config_path)?;
        let database = config.database.get_or_insert_default();
        if let Some(existing) = database.servers.get_mut(&self.name) {
            existing.apply_patch(self.conn.clone());
        } else {
            database
                .servers
                .insert(self.name.clone(), self.conn.clone());
        }
        save_config(&config_path, &config)
    }
}

pub struct EditArgs {
    pub path_config: PathConfigArgs,
    /// Server name under `database.servers.<name>`
    pub name: String,
    pub conn: DbConnConfig,
}

impl EditArgs {
    fn run(&self) -> anyhow::Result<()> {
        let config_path = self.path_config.resolve_config()?;
        validate_name(&self.name, "server")?;
        ensure_conn_payload(&self.conn)?;

        let mut config = load_config(&config_path)?;
        let database = config
            .database
            .as_mut()
            .context("global database config is missing; use `config db add` first")?;
        let existing = database.servers.get_mut(&self.name).with_context(|| {
            format!(
                "database server '{}' not found in {}",
                self.name,
                config_path.display()
            )
        })?;
        existing.apply_patch(self.conn.clone());

        save_config(&config_path, &config)
    }
}

pub struct RemoveArgs {
    pub path_config: PathConfigArgs,
    /// Server name under `database.servers.<name>`
    pub name: String,
}

impl RemoveArgs {
    fn run(&self) -> anyhow::Result<()> {
        let config_path = self.path_config.resolve_config()?;
        validate_name(&self.name, "server")?;

        let mut config = load_config(&config_path)?;
        let Some(database) = config.database.as_mut() else {
            bail!("global database config is missing");
        };

        if database.servers.remove(&self.name).is_none() {
            let name = &self.name;
            bail!("database server '{name}' not found");
        }

        if database.servers.is_empty() && database.auto_provision.is_none() {
            config.database = None;
        }

        save_config(&config_path, &config)
    }
}
