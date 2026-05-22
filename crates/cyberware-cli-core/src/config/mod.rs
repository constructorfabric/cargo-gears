use std::fs;
use std::path::Path;

use anyhow::{Context, bail};

use crate::app_config::{AppConfig, DbConnConfig};

pub mod db;
pub mod modules;

pub struct ConfigArgs {
    pub command: ConfigCommand,
}

impl ConfigArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.command.run()
    }
}

pub enum ConfigCommand {
    Mod(modules::ModulesArgs),
    Db(Box<db::DbArgs>),
}

impl ConfigCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Mod(args) => args.run(),
            Self::Db(args) => args.run(),
        }
    }
}

pub fn load_config(path: &Path) -> anyhow::Result<AppConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("can't read config file {}", path.display()))?;
    serde_saphyr::from_str(&raw).with_context(|| format!("config not valid at {}", path.display()))
}

pub fn save_config(path: &Path, config: &AppConfig) -> anyhow::Result<()> {
    let mut serialized = serde_saphyr::to_string(config).context("failed to serialize config")?;
    if !serialized.ends_with('\n') {
        serialized.push('\n');
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, serialized)
        .with_context(|| format!("can't write temp config file {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("can't replace config file {}", path.display()))
}

pub fn validate_name(value: &str, kind: &str) -> anyhow::Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("invalid {kind} name '{value}'. Use only letters, numbers, '-' and '_'");
    }
    Ok(())
}

pub fn ensure_conn_payload(conn: &DbConnConfig) -> anyhow::Result<()> {
    if conn.has_any_value() {
        return Ok(());
    }
    bail!("no database fields provided")
}
