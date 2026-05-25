mod local_modules;
mod system_modules;

pub use local_modules::LocalModulesArgs;
pub use system_modules::SystemModulesArgs;

#[derive(Debug, Eq, PartialEq)]
pub enum ListCommand {
    LocalModules(LocalModulesArgs),
    SystemModules(SystemModulesArgs),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ListArgs {
    pub command: ListCommand,
}

impl ListArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ListCommand::LocalModules(args) => args.run(),
            ListCommand::SystemModules(args) => args.run(),
        }
    }
}
