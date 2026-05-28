mod common;

use clap::Parser;
use gears_cli::Cli;
use gears_cli_core::GearsCommand;
use gears_cli_core::manifest::ManifestSelection;
use std::ffi::OsString;
use std::path::PathBuf;

use common::parse_command;

#[test]
fn parses_lint_into_core_command() {
    let command = parse_command(&[
        "gears", "lint", "--app", "app1", "--env", "dev", "--fmt", "--strict", "--clippy",
    ]);

    assert_eq!(
        command,
        GearsCommand::Lint(gears_cli_core::lint::LintArgs {
            all: false,
            path: None,
            manifest: ManifestSelection {
                manifest: PathBuf::from("Gears.toml"),
                app: "app1".to_owned(),
                env: "dev".to_owned(),
            },
            fmt: true,
            clippy: true,
            strict: true,
            dylint: false,
        })
    );
}

#[test]
fn path_parsing_does_not_change_current_directory() -> anyhow::Result<()> {
    let original_dir = std::env::current_dir()?;
    let temp_dir = tempfile::tempdir()?;

    let args = vec![
        OsString::from("gears"),
        OsString::from("lint"),
        OsString::from("-p"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--app"),
        OsString::from("app1"),
        OsString::from("--env"),
        OsString::from("dev"),
    ];
    Cli::try_parse_from(args).expect("path should parse successfully");

    // CWD must remain unchanged after parsing
    assert_eq!(std::env::current_dir()?, original_dir);
    Ok(())
}
