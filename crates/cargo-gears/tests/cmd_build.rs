mod common;

use clap::error::ErrorKind;
use common::assert_parse_error;

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
