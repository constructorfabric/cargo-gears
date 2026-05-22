use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitArgs {
    /// Path to initialize the project
    path: PathBuf,
    /// Name of the project, it's inferred from the path name if not specified
    #[arg(short = 'n', long)]
    name: Option<String>,
    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Path to a local template (instead of git)
    #[arg(long, conflicts_with_all = ["git", "subfolder", "branch"])]
    local_path: Option<String>,
    /// url to the git repo
    #[arg(
        long,
        default_value = "git@github.com:cyberfabric/cf-template-rust.git"
    )]
    git: Option<String>,
    /// Subfolder relative to the git repo
    #[arg(long, default_value = "Init")]
    subfolder: Option<String>,
    /// Branch of the git repo
    #[arg(long, default_value = "main")]
    branch: Option<String>,
    #[arg(long)]
    r#override: bool,
}

impl InitArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::init::InitArgs::from(self).run()
    }
}

impl From<InitArgs> for cyberware_cli_core::init::InitArgs {
    fn from(args: InitArgs) -> Self {
        Self {
            path: args.path,
            name: args.name,
            verbose: args.verbose,
            local_path: args.local_path,
            git: args.git,
            subfolder: args.subfolder,
            branch: args.branch,
            r#override: args.r#override,
        }
    }
}
