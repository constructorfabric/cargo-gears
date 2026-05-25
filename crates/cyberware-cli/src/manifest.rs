use crate::common::OutputFormat;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ManifestArgs {
    /// Path to the Cyberware manifest file
    #[arg(long, default_value = cyberware_cli_core::manifest::DEFAULT_MANIFEST_FILE)]
    manifest: PathBuf,
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
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

impl From<ManifestArgs> for cyberware_cli_core::manifest::ManifestArgs {
    fn from(args: ManifestArgs) -> Self {
        Self {
            manifest: args.manifest,
            command: match args.command {
                ManifestCommand::Validate(args) => {
                    cyberware_cli_core::manifest::ManifestCommand::Validate {
                        format: args.format,
                    }
                }
                ManifestCommand::Ls(args) => cyberware_cli_core::manifest::ManifestCommand::Ls {
                    format: args.format,
                },
            },
        }
    }
}
