pub use cyberware_cli_derive::HelpSchema;

pub mod app_config;
pub mod build;
pub mod common;
pub mod config;
pub mod deploy;
pub mod docs;
pub mod generate;
pub mod help;
pub mod lint;
pub mod list;
pub mod manifest;
pub mod module_parser;
pub mod run;
pub mod test;
pub mod tools;

#[derive(Debug, Eq, PartialEq)]
pub enum CyberfabricCommand {
    Generate(generate::GenerateArgs),
    Config(config::ConfigArgs),
    Docs(docs::DocsArgs),
    Help(help::HelpArgs),
    Lint(lint::LintArgs),
    List(list::ListArgs),
    Manifest(manifest::ManifestArgs),
    Test(test::TestArgs),
    Tools(tools::ToolsArgs),
    Run(run::RunArgs),
    Build(build::BuildArgs),
    Deploy(deploy::DeployArgs),
}

impl CyberfabricCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Generate(args) => args.run(),
            Self::Config(args) => args.run(),
            Self::Docs(args) => args.run(),
            Self::Help(args) => args.run(),
            Self::Lint(args) => args.run(),
            Self::List(args) => args.run(),
            Self::Manifest(args) => args.run(),
            Self::Test(args) => args.run(),
            Self::Tools(args) => args.run(),
            Self::Run(args) => args.run(),
            Self::Build(args) => args.run(),
            Self::Deploy(args) => args.run(),
        }
    }
}
