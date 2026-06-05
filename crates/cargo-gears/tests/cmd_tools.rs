mod common;

use cargo_gears::Cli;
use cargo_gears_core::GearsCommand;
use clap::Parser;

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
        GearsCommand::Tools(cargo_gears_core::tools::ToolsParams {
            all: false,
            upgrade: true,
            install: Some(vec![
                cargo_gears_core::tools::ToolName::Rustfmt,
                cargo_gears_core::tools::ToolName::Clippy,
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
