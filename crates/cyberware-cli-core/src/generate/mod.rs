pub mod config;
pub mod module;
pub mod workspace;

pub const DEFAULT_GIT_URL: &str = "git@github.com:cyberfabric/cf-template-rust.git";
pub const DEFAULT_BRANCH: &str = "main";

#[derive(Debug, Eq, PartialEq)]
pub struct GenerateArgs {
    pub command: GenerateCommand,
}

impl GenerateArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum GenerateCommand {
    Workspace(workspace::WorkspaceArgs),
    Module(module::ModuleArgs),
    Config(config::GenerateConfigArgs),
}

impl GenerateCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Workspace(args) => args.run(),
            Self::Module(args) => args.run(),
            Self::Config(args) => args.run(),
        }
    }
}
