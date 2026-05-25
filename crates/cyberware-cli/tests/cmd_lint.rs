mod common;

use clap::Parser;
use cyberware_cli::Cli;
use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::module_parser::test_utils::CWD_MUTEX;
use std::ffi::OsString;

use common::parse_command;

#[test]
fn parses_lint_into_core_command() {
    let command = parse_command(&["cyberfabric", "lint", "--fmt", "--strict", "--clippy"]);

    assert_eq!(
        command,
        CyberfabricCommand::Lint(cyberware_cli_core::lint::LintArgs {
            all: false,
            path: None,
            fmt: true,
            clippy: true,
            strict: true,
            dylint: false,
        })
    );
}

#[test]
fn path_parsing_changes_current_directory() -> anyhow::Result<()> {
    let _lock = CWD_MUTEX.lock().expect("cwd mutex should not be poisoned");
    let original_dir = std::env::current_dir()?;
    let temp_dir = tempfile::tempdir()?;

    let args = vec![
        OsString::from("cyberfabric"),
        OsString::from("lint"),
        OsString::from("-p"),
        temp_dir.path().as_os_str().to_owned(),
    ];
    let result = Cli::try_parse_from(args);
    let parsed_dir = std::env::current_dir()?;
    std::env::set_current_dir(&original_dir)?;

    result.expect("path should parse and change cwd");
    assert_eq!(parsed_dir, temp_dir.path().canonicalize()?);
    Ok(())
}
