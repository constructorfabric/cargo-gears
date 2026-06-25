use super::{load_config, save_config, validate_name};

pub mod add;
pub mod db;
pub mod remove;

#[derive(Debug, Eq, PartialEq)]
pub struct GearsParams {
    pub command: GearsCommand,
}

#[derive(Debug, Eq, PartialEq)]
pub enum GearsCommand {
    /// Add or update a gears in the gears section (upsert)
    Add(add::AddParams),
    /// Manage gears-level database config
    Db(Box<db::ModuleDbParams>),
    /// Remove a gears from the gears section
    Rm(remove::RemoveParams),
}

impl GearsParams {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            GearsCommand::Add(args) => args.run(),
            GearsCommand::Db(args) => args.run(),
            GearsCommand::Rm(args) => args.run(),
        }
    }
}

pub(super) fn validate_module_name(module: &str) -> anyhow::Result<()> {
    validate_name(module, "module")
}
