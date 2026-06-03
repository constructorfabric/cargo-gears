use cargo_gears::Cli;
use cargo_gears_core::GearsCommand;
use clap::{Parser, error::ErrorKind};

#[allow(clippy::expect_used)]
pub fn parse_command(args: &[&str]) -> GearsCommand {
    Cli::try_parse_from(args).expect("argv should parse").into()
}

// This is not dead code, it is used in tests but based on how cargo handles tests it is required
// Alternate solution: Subdirectory + main.rs — single binary, no dead code warning
// A separate test-utils crate — overkill for this case
#[allow(dead_code)]
pub fn assert_parse_error(args: &[&str], kind: ErrorKind) {
    let Err(error) = Cli::try_parse_from(args) else {
        panic!("argv should fail to parse")
    };
    assert_eq!(error.kind(), kind);
}
