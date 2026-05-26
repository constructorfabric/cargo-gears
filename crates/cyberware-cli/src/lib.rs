mod build;
mod common;
mod config;
mod deploy;
mod docs;
mod generate;
mod lint;
mod list;
mod manifest;
mod run;
mod testing;
mod tools;

#[derive(clap::Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
#[command(name = "cyberfabric")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Generate workspace, module, and config scaffolding
    Generate(generate::GenerateArgs),
    /// Alias for `generate workspace`
    New(generate::WorkspaceArgs),
    /// Utility to modify a provided configuration file
    Config(Box<config::ConfigArgs>),
    /// Utility to retrieve external dependency code in a token-friendly way
    Docs(docs::DocsArgs),
    /// Orchestrate the linting process of the project
    Lint(lint::LintArgs),
    /// Inspect workspace modules, system modules, and project state
    List(list::ListArgs),
    /// Inspect and validate Cyberware.toml manifests
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
        cyberware_cli_core::CyberfabricCommand::from(self).run()
    }
}

impl From<Cli> for cyberware_cli_core::CyberfabricCommand {
    fn from(cli: Cli) -> Self {
        match cli.command {
            Commands::Generate(generate) => Self::Generate(generate.into()),
            Commands::New(workspace) => {
                Self::Generate(cyberware_cli_core::generate::GenerateArgs {
                    command: cyberware_cli_core::generate::GenerateCommand::Workspace(
                        workspace.into(),
                    ),
                })
            }
            Commands::Config(config) => Self::Config((*config).into()),
            Commands::Docs(docs) => Self::Docs(docs.into()),
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
