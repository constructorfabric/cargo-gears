mod common;

use clap::error::ErrorKind;
use common::{assert_parse_error, parse_cli};

#[test]
fn parses_build_flags() {
    let cli = parse_cli(&[
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

    let cargo_gears::Commands::Build(args) = cli.command() else {
        panic!("expected build command");
    };

    assert!(args.build_run_args.otel);
    assert!(args.build_run_args.fips);
    assert!(args.build_run_args.release);
    assert!(args.build_run_args.clean);
    assert!(args.build_run_args.dry_run);
    assert_eq!(args.build_run_args.name.as_deref(), Some("demo-server"));
    assert_eq!(args.build_run_args.manifest.app.as_deref(), Some("app1"));
    assert_eq!(args.build_run_args.manifest.env.as_deref(), Some("dev"));
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
    let cli = parse_cli(&[
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

    let cargo_gears::Commands::Build(args) = cli.command() else {
        panic!("expected build command");
    };

    assert!(args.build_run_args.no_otel);
    assert!(args.build_run_args.no_fips);
    assert!(args.build_run_args.no_clean);
    assert!(args.build_run_args.no_release);
}
