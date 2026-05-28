mod common;

use gears_cli_core::GearsCommand;
use gears_cli_core::common::BuildRunArgs;
use gears_cli_core::manifest::ManifestSelection;
use std::path::PathBuf;

use clap::error::ErrorKind;
use common::{assert_parse_error, parse_command};

#[test]
fn parses_build_into_core_command() {
    let command = parse_command(&[
        "gears",
        "build",
        "--app",
        "app1",
        "--env",
        "dev",
        "--otel",
        "--fips",
        "--release",
        "--clean",
        "--dry-run",
        "--name",
        "demo-server",
    ]);

    assert_eq!(
        command,
        GearsCommand::Build(gears_cli_core::build::BuildArgs {
            build_run_args: BuildRunArgs {
                path: None,
                manifest: ManifestSelection {
                    manifest: PathBuf::from("Gears.toml"),
                    app: "app1".to_owned(),
                    env: "dev".to_owned(),
                },
                otel: Some(true),
                fips: Some(true),
                release: Some(true),
                clean: Some(true),
                dry_run: true,
                name: Some("demo-server".to_owned()),
            },
        })
    );
}

#[test]
fn rejects_build_positive_and_negative_boolean_pairs() {
    for pair in [
        ["--otel", "--no-otel"],
        ["--fips", "--no-fips"],
        ["--release", "--no-release"],
        ["--clean", "--no-clean"],
    ] {
        assert_parse_error(
            &[
                "gears", "build", "--app", "app1", "--env", "dev", pair[0], pair[1],
            ],
            ErrorKind::ArgumentConflict,
        );
    }
}

#[test]
fn parses_build_negative_boolean_overrides() {
    let command = parse_command(&[
        "gears",
        "build",
        "--app",
        "app1",
        "--env",
        "dev",
        "--no-otel",
        "--no-fips",
        "--no-clean",
        "--no-release",
    ]);

    let GearsCommand::Build(args) = command else {
        panic!("expected build command")
    };

    assert_eq!(args.build_run_args.otel, Some(false));
    assert_eq!(args.build_run_args.fips, Some(false));
    assert_eq!(args.build_run_args.clean, Some(false));
    assert_eq!(args.build_run_args.release, Some(false));
}
