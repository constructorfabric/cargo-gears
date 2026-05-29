mod build;
mod common;
mod config;
mod deploy;
mod generate;
mod help;
mod lint;
mod list;
mod manifest;
mod run;
mod source;
mod testing;
mod tools;

#[derive(clap::Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
#[command(name = "gears")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum Commands {
    /// Generate workspace, module, and config scaffolding
    Generate(generate::GenerateArgs),
    /// Alias for `generate workspace`
    New(generate::WorkspaceArgs),
    /// Utility to modify a provided configuration file
    Config(Box<config::ConfigArgs>),
    /// Resolve Rust source code from a crate or module path
    Src(source::SourceArgs),
    /// Schema, topic, and source-code help for developers and LLMs
    Help(help::HelpArgs),
    /// Orchestrate the linting process of the project
    Lint(lint::LintArgs),
    /// Inspect workspace modules, system modules, and project state
    List(list::ListArgs),
    /// Inspect and validate Gears.toml manifests
    Manifest(manifest::ManifestArgs),
    /// Orchestrate the testing process of the project
    Test(testing::TestArgs),
    /// Handle the required or optional tools for the project
    Tools(tools::ToolsArgs),
    /// Generate an ephemeral cargo binary based on the provided configuration file
    Run(run::RunArgs),
    /// Same as run but stops at the build step
    Build(build::BuildArgs),
    /// Build a Docker image for the generated or provided server manifest
    Deploy(deploy::DeployArgs),
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::GearsCommand::from(self).run()
    }
}

impl From<Cli> for gears_cli_core::GearsCommand {
    fn from(cli: Cli) -> Self {
        match cli.command {
            Commands::Generate(generate) => Self::Generate(generate.into()),
            Commands::New(workspace) => Self::Generate(gears_cli_core::generate::GenerateParams {
                command: gears_cli_core::generate::GenerateCommand::Workspace(workspace.into()),
            }),
            Commands::Config(config) => Self::Config((*config).into()),
            Commands::Src(src) => Self::Src(src.into()),
            Commands::Help(help) => help.into(),
            Commands::Lint(lint) => Self::Lint(lint.into()),
            Commands::List(list) => Self::List(list.into()),
            Commands::Manifest(manifest) => Self::Manifest(manifest.into()),
            Commands::Test(test) => Self::Test(test.into()),
            Commands::Tools(tools) => Self::Tools(tools.into()),
            Commands::Run(run) => Self::Run(run.into()),
            Commands::Build(build) => Self::Build(build.into()),
            Commands::Deploy(deploy) => Self::Deploy(deploy.into()),
        }
    }
}
