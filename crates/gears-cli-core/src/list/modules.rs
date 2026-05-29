use crate::common::{OutputFormat, Registry};
use std::path::PathBuf;

use super::local_modules::print_local_modules;
use super::system_modules::print_system_modules;

#[derive(Debug, Eq, PartialEq)]
pub struct ModulesParams {
    pub path: Option<PathBuf>,
    pub verbose: bool,
    pub registry: Registry,
    pub format: OutputFormat,
}

impl ModulesParams {
    pub fn run(&self) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Table => {}
            OutputFormat::Json => todo!("JSON output is not yet implemented for list modules"),
        }

        print_system_modules(self.verbose, self.registry)?;
        println!();
        print_local_modules(self.verbose, self.path.as_deref())
    }
}
