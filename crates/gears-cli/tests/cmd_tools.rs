mod common;

use clap::Parser;
use gears_cli::Cli;
use gears_cli_core::GearsCommand;

use common::parse_command;

#[test]
fn parses_tools_into_core_command() {
    let command = parse_command(&[
        "gears",
        "tools",
        "--install",
        "rustfmt,clippy",
        "--upgrade",
        "-y",
        "-v",
    ]);

    assert_eq!(
        command,
        GearsCommand::Tools(gears_cli_core::tools::ToolsArgs {
            all: false,
            upgrade: true,
            install: Some(vec![
                gears_cli_core::tools::ToolName::Rustfmt,
                gears_cli_core::tools::ToolName::Clippy,
            ]),
            yolo: true,
            verbose: true,
        })
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
