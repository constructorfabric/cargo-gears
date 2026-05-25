mod apps;
mod configs;
mod local_modules;
mod modules;
mod system_modules;

pub use apps::AppsArgs;
pub use configs::ConfigsArgs;
pub use local_modules::LocalModulesArgs;
pub use modules::ModulesArgs;
pub use system_modules::SystemModulesArgs;

#[derive(Debug, Eq, PartialEq)]
pub enum ListCommand {
    Modules(ModulesArgs),
    LocalModules(LocalModulesArgs),
    SystemModules(SystemModulesArgs),
    Configs(ConfigsArgs),
    Apps(AppsArgs),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ListArgs {
    pub command: ListCommand,
}

impl ListArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ListCommand::Modules(args) => args.run(),
            ListCommand::LocalModules(args) => args.run(),
            ListCommand::SystemModules(args) => args.run(),
            ListCommand::Configs(args) => args.run(),
            ListCommand::Apps(args) => args.run(),
        }
    }
}
