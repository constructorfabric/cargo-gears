mod common;

use clap::error::ErrorKind;
use common::assert_parse_error;
use common::parse_command;
use gears_cli_core::GearsCommand;
use gears_cli_core::common::OutputFormat;
use gears_cli_core::manifest::{ManifestArgs, ManifestCommand};
use std::path::PathBuf;

#[test]
fn parses_manifest_default_path_into_core_command() {
    let command = parse_command(&["gears", "manifest", "ls"]);

    assert_eq!(
        command,
        GearsCommand::Manifest(ManifestArgs {
            path: None,
            manifest: PathBuf::from("Gears.toml"),
            command: ManifestCommand::Ls {
                format: OutputFormat::Table,
            },
        })
    );
}

#[test]
fn parses_manifest_validate_into_core_command() {
    let command = parse_command(&[
        "gears",
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
        GearsCommand::Manifest(ManifestArgs {
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
        "gears",
        "manifest",
        "--manifest",
        "custom.toml",
        "ls",
        "--format",
        "json",
    ]);

    assert_eq!(
        command,
        GearsCommand::Manifest(ManifestArgs {
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
        &["gears", "manifest", "-p", "Cargo.toml", "ls"],
        ErrorKind::ValueValidation,
    );
}
