use cargo_gears::Cli;
use cargo_gears_core::GearsCommand;
use clap::{Parser, error::ErrorKind};

// Each integration test file is its own binary, so functions used by *other*
// test files appear dead from the perspective of files that don't call them.

/// Parse CLI args and convert into a `GearsCommand`.
/// Only valid for non-manifest-based commands (panics for Build/Run/Test/Lint).
#[allow(dead_code)]
#[allow(clippy::expect_used)]
pub fn parse_command(args: &[&str]) -> GearsCommand {
    Cli::try_parse_from(args)
        .expect("argv should parse")
        .into_command()
}

// This is not dead code, it is used in tests but based on how cargo handles tests it is required
#[allow(dead_code)]
pub fn assert_parse_error(args: &[&str], kind: ErrorKind) {
    let Err(error) = Cli::try_parse_from(args) else {
        panic!("argv should fail to parse")
    };
    assert_eq!(error.kind(), kind);
}
