mod common;

use gears_cli_core::GearsCommand;
use gears_cli_core::common::{OutputFormat, Registry};

use common::parse_command;

#[test]
fn parses_list_modules_into_core_command() {
    let command = parse_command(&[
        "gears",
        "ls",
        "modules",
        "--verbose",
        "--registry",
        "crates.io",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(gears_cli_core::list::ListParams {
            command: gears_cli_core::list::ListCommand::Modules(
                gears_cli_core::list::ModulesParams {
                    path: None,
                    verbose: true,
                    output: gears_cli_core::list::ModulesOutput::all(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}

#[test]
fn parses_list_modules_local_flag_into_core_command() {
    let command = parse_command(&["gears", "ls", "modules", "--local"]);

    assert_eq!(
        command,
        GearsCommand::List(gears_cli_core::list::ListParams {
            command: gears_cli_core::list::ListCommand::Modules(
                gears_cli_core::list::ModulesParams {
                    path: None,
                    verbose: false,
                    output: gears_cli_core::list::ModulesOutput::local(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}

#[test]
fn parses_list_modules_system_flag_into_core_command() {
    let command = parse_command(&[
        "gears",
        "ls",
        "modules",
        "--system",
        "--verbose",
        "--registry",
        "crates.io",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(gears_cli_core::list::ListParams {
            command: gears_cli_core::list::ListCommand::Modules(
                gears_cli_core::list::ModulesParams {
                    path: None,
                    verbose: true,
                    output: gears_cli_core::list::ModulesOutput::system(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}
