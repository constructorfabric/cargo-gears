use super::{load_config, save_config, validate_name};

pub mod add;
pub mod db;
pub mod remove;

#[derive(Debug, Eq, PartialEq)]
pub struct ModulesParams {
    pub command: ModulesCommand,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModulesCommand {
    /// Add or update a module in the modules section (upsert)
    Add(add::AddParams),
    /// Manage module-level database config
    Db(Box<db::ModuleDbParams>),
    /// Remove a module from the modules section
    Rm(remove::RemoveParams),
}

impl ModulesParams {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ModulesCommand::Add(args) => args.run(),
            ModulesCommand::Db(args) => args.run(),
            ModulesCommand::Rm(args) => args.run(),
        }
    }
}

pub(super) fn validate_module_name(module: &str) -> anyhow::Result<()> {
    validate_name(module, "module")
}
