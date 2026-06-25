use super::{load_config, save_config, validate_module_name};
use crate::app_config::{AppConfig, DbConnConfig, ModuleConfig};
use crate::common::PathConfigParams;
use crate::config::ensure_conn_payload;
use anyhow::{Context, bail};
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
pub struct ModuleDbParams {
    pub command: ModuleDbCommand,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModuleDbCommand {
    /// Add or update (upsert) a module-level database configuration
    Add(AddArgs),
    /// Edit a module-level database configuration
    Edit(EditArgs),
    /// Remove a module-level database configuration
    Rm(RemoveArgs),
}

impl ModuleDbParams {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ModuleDbCommand::Add(args) => args.run(),
            ModuleDbCommand::Edit(args) => args.run(),
            ModuleDbCommand::Rm(args) => args.run(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AddArgs {
    pub path_config: PathConfigParams,
    /// Module name under `gears.<module>`
    pub module: String,
    pub conn: DbConnConfig,
}

impl AddArgs {
    fn run(&self) -> anyhow::Result<()> {
        self.path_config
            .with_workspace_dir(|_workspace_path, config_path| {
                validate_module_db_payload(&self.module, &self.conn)?;

                let mut config = load_config(config_path)?;
                if !config.gears.contains_key(&self.module) {
                    bail!(
                        "module '{}' not found in {}; use `config mod add` first",
                        self.module,
                        config_path.display()
                    );
                }
                let module_cfg = get_module_cfg_mut(&mut config, &self.module, config_path)?;
                if let Some(existing) = module_cfg.database.as_mut() {
                    existing.apply_patch(self.conn.clone());
                } else {
                    module_cfg.database = Some(self.conn.clone());
                }
                save_config(config_path, &config)
            })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct EditArgs {
    pub path_config: PathConfigParams,
    /// Module name under `gears.<module>`
    pub module: String,
    pub conn: DbConnConfig,
}

impl EditArgs {
    fn run(&self) -> anyhow::Result<()> {
        self.path_config
            .with_workspace_dir(|_workspace_path, config_path| {
                validate_module_db_payload(&self.module, &self.conn)?;

                let mut config = load_config(config_path)?;
                let module_cfg = get_module_cfg_mut(&mut config, &self.module, config_path)?;
                let db_cfg = module_cfg.database.as_mut().with_context(|| {
                    format!(
                        "module '{}' has no database config; use `config mod db add` first",
                        self.module
                    )
                })?;
                db_cfg.apply_patch(self.conn.clone());

                save_config(config_path, &config)
            })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RemoveArgs {
    pub path_config: PathConfigParams,
    /// Module name under `gears.<module>`
    pub module: String,
}

impl RemoveArgs {
    fn run(&self) -> anyhow::Result<()> {
        self.path_config
            .with_workspace_dir(|_workspace_path, config_path| {
                validate_module_name(&self.module)?;

                let mut config = load_config(config_path)?;
                let module_cfg = get_module_cfg_mut(&mut config, &self.module, config_path)?;
                if module_cfg.database.take().is_none() {
                    let module = &self.module;
                    bail!("module '{module}' has no database config");
                }

                save_config(config_path, &config)
            })
    }
}

fn get_module_cfg_mut<'a>(
    config: &'a mut AppConfig,
    module: &str,
    config_path: &Path,
) -> anyhow::Result<&'a mut ModuleConfig> {
    config
        .gears
        .get_mut(module)
        .with_context(|| format!("module '{module}' not found in {}", config_path.display()))
}

fn validate_module_db_payload(module: &str, conn: &DbConnConfig) -> anyhow::Result<()> {
    validate_module_name(module)?;
    ensure_conn_payload(conn)
}
