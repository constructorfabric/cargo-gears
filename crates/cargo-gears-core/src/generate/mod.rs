pub mod config;
pub mod gear;
pub mod workspace;

pub const DEFAULT_GIT_URL: &str = "git@github.com:Bechma/cf-template-rust.git";
pub const DEFAULT_BRANCH: &str = "main";

#[derive(Debug, Eq, PartialEq)]
pub struct GenerateParams {
    pub command: GenerateCommand,
}

impl GenerateParams {
    pub fn run(&self) -> anyhow::Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum GenerateCommand {
    Workspace(workspace::WorkspaceParams),
    Gear(gear::GearParams),
    Config(config::GenerateConfigParams),
}

impl GenerateCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Workspace(args) => args.run(),
            Self::Gear(args) => args.run(),
            Self::Config(args) => args.run(),
        }
    }
}
