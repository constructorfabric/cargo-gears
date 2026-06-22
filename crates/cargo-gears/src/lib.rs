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

use std::convert::TryFrom;

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
enum Commands {
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
    #[command(name = "ls")]
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
        match self.command {
            // Manifest-based commands: resolve CLI overrides, then run core logic.
            Commands::Lint(lint) => lint.resolve()?.run(),
            Commands::Test(test) => test.resolve()?.run(),
            Commands::Build(build) => build.resolve()?.run(),
            Commands::Run(run) => run.resolve_and_run(),
            // Non-manifest commands: pass through to core.
            other => cargo_gears_core::GearsCommand::try_from(other)?.run(),
        }
    }
}

impl TryFrom<Commands> for cargo_gears_core::GearsCommand {
    type Error = anyhow::Error;

    fn try_from(cmd: Commands) -> Result<Self, Self::Error> {
        match cmd {
            Commands::Generate(generate) => Ok(Self::Generate(generate.into())),
            Commands::New(workspace) => {
                Ok(Self::Generate(cargo_gears_core::generate::GenerateParams {
                    command: cargo_gears_core::generate::GenerateCommand::Workspace(
                        workspace.into(),
                    ),
                }))
            }
            Commands::Config(config) => Ok(Self::Config((*config).into())),
            Commands::Src(src) => Ok(Self::Src(src.into())),
            Commands::Help(help) => Ok(help.into()),
            Commands::List(list) => Ok(Self::List(list.into())),
            Commands::Manifest(manifest) => Ok(Self::Manifest(manifest.into())),
            Commands::Tools(tools) => Ok(Self::Tools(tools.into())),
            Commands::Deploy(deploy) => Ok(Self::Deploy(deploy.into())),
            // Manifest-based commands should be resolved in Cli::run(), not converted here.
            Commands::Lint(_) | Commands::Test(_) | Commands::Build(_) | Commands::Run(_) => {
                anyhow::bail!("manifest-based commands should be resolved in Cli::run()")
            }
        }
    }
}

impl TryFrom<Cli> for cargo_gears_core::GearsCommand {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        Self::try_from(cli.command)
    }
}
