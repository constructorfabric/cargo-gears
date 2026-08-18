mod common;

use cargo_gears::Cli;
use cargo_gears_core::GearsCommand;
use clap::Parser;

use common::parse_command;

#[test]
fn try_from_returns_error_for_tools_command() {
    use std::convert::TryFrom;

    let cli = Cli::try_parse_from(["gears", "tools", "--install", "rustfmt,clippy", "--upgrade"])
        .expect("should parse");
    let result = GearsCommand::try_from(cli);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "command should be dispatched directly in Cli::run()"
    );
}

#[test]
fn rejects_conflicting_tool_selection() {
    let result = Cli::try_parse_from(["gears", "tools", "--all", "--install", "rustfmt"]);
    let Err(error) = result else {
        panic!("conflicting tool selection should fail");
    };

    assert!(error.to_string().contains("cannot be used with"));
}
