use crate::common::{ManifestPath, OutputFormat, WorkspacePath};
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ManifestArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    #[command(flatten)]
    manifest_path: ManifestPath,
    #[command(subcommand)]
    command: ManifestCommand,
}

#[derive(Subcommand)]
enum ManifestCommand {
    /// Validate the manifest and referenced modules
    Validate(ManifestFormatArgs),
    /// List manifest app/environment entries
    Ls(ManifestFormatArgs),
}

#[derive(Args)]
struct ManifestFormatArgs {
    /// Workspace path (positional, takes precedence over parent -p)
    path: Option<PathBuf>,
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

impl From<ManifestArgs> for gears_cli_core::manifest::ManifestParams {
    fn from(args: ManifestArgs) -> Self {
        let parent_path = args.workspace.path;
        let (path, command) = match args.command {
            ManifestCommand::Validate(sub) => (
                sub.path.or(parent_path),
                gears_cli_core::manifest::ManifestCommand::Validate { format: sub.format },
            ),
            ManifestCommand::Ls(sub) => (
                sub.path.or(parent_path),
                gears_cli_core::manifest::ManifestCommand::Ls { format: sub.format },
            ),
        };
        Self {
            path,
            manifest: args.manifest_path.manifest,
            command,
        }
    }
}
