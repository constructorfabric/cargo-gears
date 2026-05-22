use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Args)]
pub struct ModArgs {
    #[command(subcommand)]
    command: ModCommand,
}

impl ModArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::r#mod::ModArgs {
            command: self.command.into(),
        }
        .run()
    }
}

#[derive(Subcommand)]
pub enum ModCommand {
    Add(ModuleAddArgs),
}

impl From<ModCommand> for cyberware_cli_core::r#mod::ModCommand {
    fn from(command: ModCommand) -> Self {
        match command {
            ModCommand::Add(args) => Self::Add(args.into()),
        }
    }
}

#[derive(Args)]
pub struct ModuleAddArgs {
    /// Module template and module name to generate
    #[arg(value_enum)]
    name: ModuleTemplateName,
    /// Path to the workspace root (defaults to current directory)
    #[arg(short = 'p', long, default_value = ".")]
    path: PathBuf,
    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Path to a local template (instead of git)
    #[arg(long, conflicts_with_all = ["git", "branch"])]
    local_path: Option<String>,
    /// URL to the git repo
    #[arg(
        long,
        default_value = "https://github.com/cyberfabric/cf-template-rust"
    )]
    git: Option<String>,
    /// Subfolder relative to the git repo
    #[arg(long, default_value = "Modules")]
    subfolder: String,
    /// Branch of the git repo
    #[arg(long, default_value = "main")]
    branch: Option<String>,
}

impl From<ModuleAddArgs> for cyberware_cli_core::r#mod::add::AddArgs {
    fn from(args: ModuleAddArgs) -> Self {
        Self {
            name: args.name.into(),
            path: args.path,
            verbose: args.verbose,
            local_path: args.local_path,
            git: args.git,
            subfolder: args.subfolder,
            branch: args.branch,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
enum ModuleTemplateName {
    #[value(name = "background-worker")]
    BackgroundWorker,
    #[value(name = "api-db-handler")]
    ApiDbHandler,
    #[value(name = "rest-gateway")]
    RestGateway,
}

impl From<ModuleTemplateName> for cyberware_cli_core::r#mod::add::ModuleTemplateName {
    fn from(name: ModuleTemplateName) -> Self {
        match name {
            ModuleTemplateName::BackgroundWorker => Self::BackgroundWorker,
            ModuleTemplateName::ApiDbHandler => Self::ApiDbHandler,
            ModuleTemplateName::RestGateway => Self::RestGateway,
        }
    }
}
