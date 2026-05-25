mod common;

use common::parse_command;
use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::common::OutputFormat;
use cyberware_cli_core::manifest::{ManifestArgs, ManifestCommand};
use std::path::PathBuf;

#[test]
fn parses_manifest_default_path_into_core_command() {
    let command = parse_command(&["cyberfabric", "manifest", "ls"]);

    assert_eq!(
        command,
        CyberfabricCommand::Manifest(ManifestArgs {
            manifest: PathBuf::from("Cyberware.toml"),
            command: ManifestCommand::Ls {
                format: OutputFormat::Table,
            },
        })
    );
}

#[test]
fn parses_manifest_validate_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "manifest",
        "--manifest",
        "custom.toml",
        "validate",
        "--format",
        "json",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Manifest(ManifestArgs {
            manifest: PathBuf::from("custom.toml"),
            command: ManifestCommand::Validate {
                format: OutputFormat::Json,
            },
        })
    );
}

#[test]
fn parses_manifest_ls_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "manifest",
        "--manifest",
        "custom.toml",
        "ls",
        "--format",
        "json",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Manifest(ManifestArgs {
            manifest: PathBuf::from("custom.toml"),
            command: ManifestCommand::Ls {
                format: OutputFormat::Json,
            },
        })
    );
}
