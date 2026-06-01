mod common;

use clap::error::ErrorKind;
use common::{assert_parse_error, parse_command};
use gears_cli_core::GearsCommand;
use gears_cli_core::manifest::{ManifestSelection, TestRunner};
use std::path::PathBuf;

#[test]
fn parses_test_into_core_command() {
    let command = parse_command(&[
        "gears",
        "test",
        "--manifest",
        "Gears.dev.toml",
        "--app",
        "app1",
        "--env",
        "dev",
        "--runner",
        "nextest",
        "--module",
        "module-a",
        "--coverage",
    ]);

    assert_eq!(
        command,
        GearsCommand::Test(gears_cli_core::test::TestParams {
            path: None,
            manifest: ManifestSelection {
                manifest: PathBuf::from("Gears.dev.toml"),
                app: Some("app1".to_owned()),
                env: Some("dev".to_owned()),
            },
            runner: Some(TestRunner::Nextest),
            module: Some("module-a".to_owned()),
            coverage: true,
        })
    );
}

#[test]
fn rejects_removed_no_coverage_flag() {
    assert_parse_error(
        &[
            "gears",
            "test",
            "--app",
            "app1",
            "--env",
            "dev",
            "--no-coverage",
        ],
        ErrorKind::UnknownArgument,
    );
}
