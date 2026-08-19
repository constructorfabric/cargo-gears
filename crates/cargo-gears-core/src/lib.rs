pub mod app_config;
pub mod build;
pub mod clean;
pub mod common;
pub mod config;
pub mod deploy;
pub mod gears_parser;
pub mod generate;
pub mod help;
pub mod lint;
pub mod list;
pub mod manifest;
pub mod packages;
pub mod run;
pub mod source;
pub mod test;
pub mod tools;

/// Commands that do not require manifest resolution.
///
/// Manifest-based commands (Build, Run, Test, Lint, Deploy) are resolved and
/// dispatched directly in the CLI layer — see `Cli::run()`.
#[derive(Debug, Eq, PartialEq)]
pub enum GearsCommand {
    Generate(generate::GenerateParams),
    Config(config::ConfigParams),
    Src(source::SourceParams),
    Help(help::HelpParams),
    List(list::ListParams),
    Manifest(manifest::ManifestParams),
    Tools(tools::ToolsParams),
}

impl GearsCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Generate(args) => args.run(),
            Self::Config(args) => args.run(),
            Self::Src(args) => args.run(),
            Self::Help(args) => args.run(),
            Self::List(args) => args.run(),
            Self::Manifest(args) => args.run(),
            Self::Tools(args) => args.run(),
        }
    }
}
