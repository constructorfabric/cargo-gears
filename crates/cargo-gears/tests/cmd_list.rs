mod common;

use cargo_gears_core::GearsCommand;
use cargo_gears_core::common::{OutputFormat, Registry};

use common::parse_command;

#[test]
fn parses_list_modules_into_core_command() {
    let command = parse_command(&[
        "gears",
        "ls",
        "gears",
        "--verbose",
        "--registry",
        "crates.io",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Gears(
                cargo_gears_core::list::GearsParams {
                    path: None,
                    verbose: true,
                    output: cargo_gears_core::list::GearsOutput::all(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Json,
                },
            ),
        })
    );
}

#[test]
fn parses_list_modules_local_flag_into_core_command() {
    let command = parse_command(&["gears", "ls", "gears", "--local"]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Gears(
                cargo_gears_core::list::GearsParams {
                    path: None,
                    verbose: false,
                    output: cargo_gears_core::list::GearsOutput::local(),
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
        "gears",
        "--system",
        "--verbose",
        "--registry",
        "crates.io",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Gears(
                cargo_gears_core::list::GearsParams {
                    path: None,
                    verbose: true,
                    output: cargo_gears_core::list::GearsOutput::system(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::Json,
                },
            ),
        })
    );
}
