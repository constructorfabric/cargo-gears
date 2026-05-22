mod build;
mod common;
mod config;
mod deploy;
mod docs;
mod init;
mod lint;
mod module;
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
    /// Initialize a new project in a non-existing folder
    Init(init::InitArgs),
    /// Add new modules to an existing project
    Mod(module::ModArgs),
    /// Utility to modify a provided configuration file
    Config(Box<config::ConfigArgs>),
    /// Utility to retrieve external dependency code in a token-friendly way
    Docs(docs::DocsArgs),
    /// Orchestrate the linting process of the project
    Lint(lint::LintArgs),
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
            Commands::Init(init) => init.run(),
            Commands::Mod(r#mod) => r#mod.run(),
            Commands::Config(config) => config.run(),
            Commands::Docs(docs) => docs.run(),
            Commands::Lint(lint) => lint.run(),
            Commands::Test(test) => test.run(),
            Commands::Tools(tools) => tools.run(),
            Commands::Run(run) => run.run(),
            Commands::Build(build) => build.run(),
            Commands::Deploy(deploy) => deploy.run(),
        }
    }
}
