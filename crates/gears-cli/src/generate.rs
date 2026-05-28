use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    command: GenerateCommand,
}

impl GenerateArgs {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::generate::GenerateArgs::from(self).run()
    }
}

impl From<GenerateArgs> for gears_cli_core::generate::GenerateArgs {
    fn from(args: GenerateArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Workspace
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct WorkspaceArgs {
    /// Path to initialize the project
    path: PathBuf,
    /// Template name (defaults to "default")
    #[arg(short = 't', long, default_value = "default")]
    template: String,
    /// Name of the project, it's inferred from the path name if not specified
    #[arg(short = 'p', long)]
    project: Option<String>,
    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Path to a local template directory
    #[arg(long, conflicts_with_all = ["git", "subfolder", "branch"])]
    local_path: Option<String>,
    /// URL to the git repo
    #[arg(long)]
    git: Option<String>,
    /// Subfolder relative to the git repo
    #[arg(long)]
    subfolder: Option<String>,
    /// Branch of the git repo
    #[arg(long)]
    branch: Option<String>,
    #[arg(long)]
    r#override: bool,
}

impl WorkspaceArgs {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::generate::workspace::WorkspaceArgs::from(self).run()
    }
}

impl From<WorkspaceArgs> for gears_cli_core::generate::workspace::WorkspaceArgs {
    fn from(args: WorkspaceArgs) -> Self {
        Self {
            path: args.path,
            template: args.template,
            name: args.project,
            verbose: args.verbose,
            local_path: args.local_path,
            git: args.git,
            subfolder: args.subfolder,
            branch: args.branch,
            r#override: args.r#override,
        }
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct ModuleArgs {
    /// Template name (e.g. background-worker, api-db-handler, api-gateway)
    #[arg(short = 't', long)]
    template: String,
    /// Module name; defaults to the template name when absent
    #[arg(short = 'n', long)]
    name: Option<String>,
    /// Path to the workspace root (defaults to current directory)
    #[arg(short = 'p', long, default_value = ".")]
    path: PathBuf,
    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Path to a local template directory
    #[arg(long, conflicts_with_all = ["git", "branch"])]
    local_path: Option<String>,
    /// URL to the git repo
    #[arg(long)]
    git: Option<String>,
    /// Subfolder relative to the git repo
    #[arg(long)]
    subfolder: Option<String>,
    /// Branch of the git repo
    #[arg(long)]
    branch: Option<String>,
}

impl ModuleArgs {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::generate::module::ModuleArgs::from(self).run()
    }
}

impl From<ModuleArgs> for gears_cli_core::generate::module::ModuleArgs {
    fn from(args: ModuleArgs) -> Self {
        Self {
            template: args.template,
            name: args.name,
            path: args.path,
            verbose: args.verbose,
            local_path: args.local_path,
            git: args.git,
            subfolder: args.subfolder,
            branch: args.branch,
        }
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct GenerateConfigArgs {
    /// Config template to use: dev, prod, or db
    #[arg(short = 't', long)]
    template: String,
    /// Application name for the output filename
    #[arg(long)]
    app: Option<String>,
    /// Environment name for the output filename
    #[arg(long)]
    env: Option<String>,
    /// Custom output filename (overrides app/env naming)
    #[arg(long)]
    name: Option<String>,
    /// Workspace root path (defaults to current directory)
    #[arg(short = 'p', long, default_value = ".")]
    path: PathBuf,
}

impl GenerateConfigArgs {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::generate::config::GenerateConfigArgs::from(self).run()
    }
}

impl From<GenerateConfigArgs> for gears_cli_core::generate::config::GenerateConfigArgs {
    fn from(args: GenerateConfigArgs) -> Self {
        Self {
            template: args.template,
            app: args.app,
            env: args.env,
            name: args.name,
            path: args.path,
        }
    }
}

// ---------------------------------------------------------------------------
// Subcommand enum
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum GenerateCommand {
    /// Generate a new Gears workspace
    Workspace(WorkspaceArgs),
    /// Generate a new module from a template
    Module(ModuleArgs),
    /// Generate a runtime configuration file from a template
    Config(GenerateConfigArgs),
}

impl From<GenerateCommand> for gears_cli_core::generate::GenerateCommand {
    fn from(command: GenerateCommand) -> Self {
        match command {
            GenerateCommand::Workspace(args) => Self::Workspace(args.into()),
            GenerateCommand::Module(args) => Self::Module(args.into()),
            GenerateCommand::Config(args) => Self::Config(args.into()),
        }
    }
}
