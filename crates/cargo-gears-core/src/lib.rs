pub use cargo_gears_macros::HelpSchema;

pub mod app_config;
pub mod build;
pub mod common;
pub mod config;
pub mod deploy;
pub mod generate;
pub mod help;
pub mod lint;
pub mod list;
pub mod manifest;
pub mod module_parser;
pub mod run;
pub mod source;
pub mod test;
pub mod tools;

#[derive(Debug, Eq, PartialEq)]
pub enum GearsCommand {
    Generate(generate::GenerateParams),
    Config(config::ConfigParams),
    Src(source::SourceParams),
    Help(help::HelpParams),
    Lint(lint::LintParams),
    List(list::ListParams),
    Manifest(manifest::ManifestParams),
    Test(test::TestParams),
    Tools(tools::ToolsParams),
    Run(run::RunParams),
    Build(build::BuildParams),
    Deploy(deploy::DeployParams),
}

impl GearsCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Generate(args) => args.run(),
            Self::Config(args) => args.run(),
            Self::Src(args) => args.run(),
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
