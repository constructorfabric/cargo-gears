mod build;
mod clean;
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
    /// Check that cargo-gears satisfies a version requirement (e.g. '>=0.0.3')
    /// and exit with code 0 (satisfied) or 1 (not satisfied).
    #[arg(long, value_name = "REQ")]
    check_version: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
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
    /// Inspect workspace gears, system gears, and project state
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
    /// Remove the generated server project and its workspace member entry
    Clean(clean::CleanArgs),
    /// Build a Docker image for the generated or provided server manifest
    Deploy(deploy::DeployArgs),
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        if let Some(req) = &self.check_version {
            return Self::run_check_version(req);
        }

        let Some(command) = self.command else {
            anyhow::bail!("no subcommand provided. Run 'cargo gears --help' for usage.");
        };

        match command {
            // Manifest-based commands: resolve CLI overrides, then run core logic.
            Commands::Lint(lint) => lint.resolve()?.run(),
            Commands::Test(test) => test.resolve()?.run(),
            Commands::Build(build) => build.resolve()?.run(),
            Commands::Clean(clean) => clean.resolve()?.run(),
            Commands::Run(run) => run.resolve_and_run(),
            // Non-manifest commands: pass through to core.
            other => cargo_gears_core::GearsCommand::try_from(other)?.run(),
        }
    }

    fn run_check_version(req: &str) -> anyhow::Result<()> {
        let current = env!("CARGO_PKG_VERSION");
        let version = semver::Version::parse(current)
            .map_err(|e| anyhow::anyhow!("cannot parse own version '{}': {}", current, e))?;
        let requirement = semver::VersionReq::parse(req)
            .map_err(|e| anyhow::anyhow!("invalid version requirement '{}': {}", req, e))?;
        if requirement.matches(&version) {
            println!("{}", current);
            Ok(())
        } else {
            eprintln!("cargo-gears {} does not satisfy {}", current, req);
            std::process::exit(1);
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
            Commands::Lint(_)
            | Commands::Test(_)
            | Commands::Build(_)
            | Commands::Clean(_)
            | Commands::Run(_) => {
                anyhow::bail!("manifest-based commands should be resolved in Cli::run()")
            }
        }
    }
}

impl TryFrom<Cli> for cargo_gears_core::GearsCommand {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        match cli.command {
            Some(cmd) => Self::try_from(cmd),
            None => anyhow::bail!("no subcommand provided"),
        }
    }
}
