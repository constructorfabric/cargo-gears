mod common;

use cargo_gears_core::GearsCommand;
use cargo_gears_core::common::BuildRunParams;
use cargo_gears_core::manifest::ManifestSelection;
use std::path::PathBuf;

use clap::error::ErrorKind;
use common::{assert_parse_error, parse_command};

#[test]
fn parses_run_into_core_command() {
    let command = parse_command(&[
        "gears",
        "run",
        "--watch",
        "--app",
        "app1",
        "--env",
        "dev",
        "--release",
        "--dry-run",
    ]);

    assert_eq!(
        command,
        GearsCommand::Run(cargo_gears_core::run::RunParams {
            watch: Some(true),
            br_args: BuildRunParams {
                path: None,
                manifest: ManifestSelection {
                    manifest: PathBuf::from("Gears.toml"),
                    app: Some("app1".to_owned()),
                    env: Some("dev".to_owned()),
                },
                otel: None,
                fips: None,
                release: Some(true),
                clean: None,
                dry_run: true,
                name: None,
            },
        })
    );
}

#[test]
fn rejects_run_positive_and_negative_boolean_pairs() {
    for pair in [
        ["--watch", "--no-watch"],
        ["--otel", "--no-otel"],
        ["--fips", "--no-fips"],
        ["--release", "--no-release"],
        ["--clean", "--no-clean"],
    ] {
        assert_parse_error(
            &[
                "gears", "run", "--app", "app1", "--env", "dev", pair[0], pair[1],
            ],
            ErrorKind::ArgumentConflict,
        );
    }
}

#[test]
fn parses_run_negative_boolean_overrides() {
    let command = parse_command(&[
        "gears",
        "run",
        "--app",
        "app1",
        "--env",
        "dev",
        "--no-watch",
        "--no-otel",
        "--no-fips",
        "--no-release",
        "--no-clean",
    ]);

    let GearsCommand::Run(args) = command else {
        panic!("expected run command")
    };

    assert_eq!(args.watch, Some(false));
    assert_eq!(args.br_args.otel, Some(false));
    assert_eq!(args.br_args.fips, Some(false));
    assert_eq!(args.br_args.release, Some(false));
    assert_eq!(args.br_args.clean, Some(false));
}

#[test]
fn parses_run_without_app_and_env() {
    let command = parse_command(&["gears", "run"]);

    let GearsCommand::Run(args) = command else {
        panic!("expected run command")
    };

    assert_eq!(args.br_args.manifest.app, None);
    assert_eq!(args.br_args.manifest.env, None);
}
