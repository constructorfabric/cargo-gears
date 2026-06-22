mod common;

use clap::Parser;
use clap::error::ErrorKind;
use common::assert_parse_error;

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
fn try_from_returns_error_for_run_command() {
    use cargo_gears::Cli;
    use cargo_gears_core::GearsCommand;
    use std::convert::TryFrom;

    let cli = Cli::try_parse_from(["gears", "run", "--app", "app1", "--env", "dev"])
        .expect("should parse");
    let result = GearsCommand::try_from(cli);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "manifest-based commands should be resolved in Cli::run()"
    );
}
