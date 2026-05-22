use super::{load_config, resolve_modules_context, save_config, validate_module_name};
use crate::app_config::{AppConfig, DbConnConfig, ModuleConfig};
use crate::common::PathConfigArgs;
use crate::config::ensure_conn_payload;
use anyhow::{Context, bail};
use std::path::Path;

pub struct ModuleDbArgs {
    pub command: ModuleDbCommand,
}

pub enum ModuleDbCommand {
    /// Add or update (upsert) a module-level database configuration
    Add(AddArgs),
    /// Edit a module-level database configuration
    Edit(EditArgs),
    /// Remove a module-level database configuration
    Rm(RemoveArgs),
}

impl ModuleDbArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ModuleDbCommand::Add(args) => args.run(),
            ModuleDbCommand::Edit(args) => args.run(),
            ModuleDbCommand::Rm(args) => args.run(),
        }
    }
}

pub struct AddArgs {
    pub path_config: PathConfigArgs,
    /// Module name under `modules.<module>`
    pub module: String,
    pub conn: DbConnConfig,
}

impl AddArgs {
    fn run(&self) -> anyhow::Result<()> {
        validate_module_db_payload(&self.module, &self.conn)?;

        let context = resolve_modules_context(&self.path_config)?;
        let mut config = load_config(&context.config_path)?;
        if !config.modules.contains_key(&self.module) {
            bail!(
                "module '{}' not found in {}; use `config mod add` first",
                self.module,
                context.config_path.display()
            );
        }
        let module_cfg = get_module_cfg_mut(&mut config, &self.module, &context.config_path)?;
        if let Some(existing) = module_cfg.database.as_mut() {
            existing.apply_patch(self.conn.clone());
        } else {
            module_cfg.database = Some(self.conn.clone());
        }
        save_config(&context.config_path, &config)
    }
}

pub struct EditArgs {
    pub path_config: PathConfigArgs,
    /// Module name under `modules.<module>`
    pub module: String,
    pub conn: DbConnConfig,
}

impl EditArgs {
    fn run(&self) -> anyhow::Result<()> {
        validate_module_db_payload(&self.module, &self.conn)?;

        let context = resolve_modules_context(&self.path_config)?;
        let mut config = load_config(&context.config_path)?;
        let module_cfg = get_module_cfg_mut(&mut config, &self.module, &context.config_path)?;
        let db_cfg = module_cfg.database.as_mut().with_context(|| {
            format!(
                "module '{}' has no database config; use `config mod db add` first",
                self.module
            )
        })?;
        db_cfg.apply_patch(self.conn.clone());

        save_config(&context.config_path, &config)
    }
}

pub struct RemoveArgs {
    pub path_config: PathConfigArgs,
    /// Module name under `modules.<module>`
    pub module: String,
}

impl RemoveArgs {
    fn run(&self) -> anyhow::Result<()> {
        validate_module_name(&self.module)?;

        let context = resolve_modules_context(&self.path_config)?;
        let mut config = load_config(&context.config_path)?;
        let module_cfg = get_module_cfg_mut(&mut config, &self.module, &context.config_path)?;
        if module_cfg.database.take().is_none() {
            let module = &self.module;
            bail!("module '{module}' has no database config");
        }

        save_config(&context.config_path, &config)
    }
}

fn get_module_cfg_mut<'a>(
    config: &'a mut AppConfig,
    module: &str,
    config_path: &Path,
) -> anyhow::Result<&'a mut ModuleConfig> {
    config
        .modules
        .get_mut(module)
        .with_context(|| format!("module '{module}' not found in {}", config_path.display()))
}

fn validate_module_db_payload(module: &str, conn: &DbConnConfig) -> anyhow::Result<()> {
    validate_module_name(module)?;
    ensure_conn_payload(conn)
}
