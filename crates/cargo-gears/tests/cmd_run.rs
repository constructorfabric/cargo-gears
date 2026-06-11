mod common;

use clap::error::ErrorKind;
use common::{assert_parse_error, parse_cli};

#[test]
fn parses_run_flags() {
    let cli = parse_cli(&[
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

    let cargo_gears::Commands::Run(args) = cli.command() else {
        panic!("expected run command");
    };

    assert!(args.watch);
    assert!(args.br_args.release);
    assert!(args.br_args.dry_run);
    assert_eq!(args.br_args.manifest.app.as_deref(), Some("app1"));
    assert_eq!(args.br_args.manifest.env.as_deref(), Some("dev"));
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
    let cli = parse_cli(&[
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

    let cargo_gears::Commands::Run(args) = cli.command() else {
        panic!("expected run command");
    };

    assert!(args.no_watch);
    assert!(args.br_args.no_otel);
    assert!(args.br_args.no_fips);
    assert!(args.br_args.no_release);
    assert!(args.br_args.no_clean);
}

#[test]
fn parses_run_without_app_and_env() {
    let cli = parse_cli(&["gears", "run"]);

    let cargo_gears::Commands::Run(args) = cli.command() else {
        panic!("expected run command");
    };

    assert_eq!(args.br_args.manifest.app, None);
    assert_eq!(args.br_args.manifest.env, None);
}
