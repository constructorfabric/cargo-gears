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
            other => other.into_command().run(),
        }
    }

    /// Convert into a `GearsCommand` for non-manifest-based commands.
    ///
    /// Panics if called on a manifest-based command (Build, Run, Test, Lint).
    #[must_use]
    pub fn into_command(self) -> cargo_gears_core::GearsCommand {
        self.command.into_command()
    }
}

impl Commands {
    fn into_command(self) -> cargo_gears_core::GearsCommand {
        match self {
            Self::Generate(generate) => cargo_gears_core::GearsCommand::Generate(generate.into()),
            Self::New(workspace) => cargo_gears_core::GearsCommand::Generate(
                cargo_gears_core::generate::GenerateParams {
                    command: cargo_gears_core::generate::GenerateCommand::Workspace(
                        workspace.into(),
                    ),
                },
            ),
            Self::Config(config) => cargo_gears_core::GearsCommand::Config((*config).into()),
            Self::Src(src) => cargo_gears_core::GearsCommand::Src(src.into()),
            Self::Help(help) => help.into(),
            Self::List(list) => cargo_gears_core::GearsCommand::List(list.into()),
            Self::Manifest(manifest) => cargo_gears_core::GearsCommand::Manifest(manifest.into()),
            Self::Tools(tools) => cargo_gears_core::GearsCommand::Tools(tools.into()),
            Self::Deploy(deploy) => cargo_gears_core::GearsCommand::Deploy(deploy.into()),
            // Manifest-based commands handled in Cli::run() — unreachable here.
            Self::Lint(_) | Self::Test(_) | Self::Build(_) | Self::Run(_) => {
                unreachable!("manifest-based commands are resolved in Cli::run()")
            }
        }
    }
}
