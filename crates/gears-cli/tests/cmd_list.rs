mod common;

use gears_cli_core::GearsCommand;
use gears_cli_core::common::{OutputFormat, Registry};

use common::parse_command;

#[test]
fn parses_list_modules_into_core_command() {
    let command = parse_command(&[
        "gears",
        "list",
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
                    registry: Registry::CratesIo,
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}

#[test]
fn parses_list_local_modules_into_core_command() {
    let command = parse_command(&["gears", "list", "local-modules", "--verbose"]);

    assert_eq!(
        command,
        GearsCommand::List(gears_cli_core::list::ListParams {
            command: gears_cli_core::list::ListCommand::LocalModules(
                gears_cli_core::list::LocalModulesParams {
                    path: None,
                    verbose: true,
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}

#[test]
fn parses_list_system_modules_into_core_command() {
    let command = parse_command(&[
        "gears",
        "list",
        "system-modules",
        "--verbose",
        "--registry",
        "crates.io",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(gears_cli_core::list::ListParams {
            command: gears_cli_core::list::ListCommand::SystemModules(
                gears_cli_core::list::SystemModulesParams {
                    verbose: true,
                    registry: Registry::CratesIo,
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}

#[test]
fn parses_list_configs_into_core_command() {
    let command = parse_command(&["gears", "list", "configs"]);

    assert_eq!(
        command,
        GearsCommand::List(gears_cli_core::list::ListParams {
            command: gears_cli_core::list::ListCommand::Configs(
                gears_cli_core::list::ConfigsParams {
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}

#[test]
fn parses_list_apps_into_core_command() {
    let command = parse_command(&["gears", "list", "apps"]);

    assert_eq!(
        command,
        GearsCommand::List(gears_cli_core::list::ListParams {
            command: gears_cli_core::list::ListCommand::Apps(gears_cli_core::list::AppsParams {
                format: OutputFormat::Table,
            },),
        })
    );
}
