mod common;

use cargo_gears_core::GearsCommand;
use cargo_gears_core::common::{OutputFormat, Registry};

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
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Modules(
                cargo_gears_core::list::ModulesParams {
                    path: None,
                    verbose: true,
                    output: cargo_gears_core::list::ModulesOutput::all(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Json,
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
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Modules(
                cargo_gears_core::list::ModulesParams {
                    path: None,
                    verbose: false,
                    output: cargo_gears_core::list::ModulesOutput::local(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Json,
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
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Modules(
                cargo_gears_core::list::ModulesParams {
                    path: None,
                    verbose: true,
                    output: cargo_gears_core::list::ModulesOutput::system(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Json,
                },
            ),
        })
    );
}
