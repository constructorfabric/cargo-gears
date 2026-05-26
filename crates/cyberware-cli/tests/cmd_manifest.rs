mod common;

use clap::error::ErrorKind;
use common::assert_parse_error;
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
            path: None,
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
        "-p",
        ".",
        "--manifest",
        "custom.toml",
        "validate",
        "--format",
        "json",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Manifest(ManifestArgs {
            path: Some(PathBuf::from(".").canonicalize().unwrap()),
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
            path: None,
            manifest: PathBuf::from("custom.toml"),
            command: ManifestCommand::Ls {
                format: OutputFormat::Json,
            },
        })
    );
}

#[test]
fn rejects_manifest_path_when_p_is_not_a_directory() {
    assert_parse_error(
        &["cyberfabric", "manifest", "-p", "Cargo.toml", "ls"],
        ErrorKind::ValueValidation,
    );
}
