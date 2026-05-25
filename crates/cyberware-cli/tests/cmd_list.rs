mod common;

use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::common::{OutputFormat, Registry};

use common::parse_command;

#[test]
fn parses_list_modules_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "list",
        "modules",
        "--verbose",
        "--registry",
        "crates.io",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::List(cyberware_cli_core::list::ListArgs {
            command: cyberware_cli_core::list::ListCommand::Modules(
                cyberware_cli_core::list::ModulesArgs {
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
    let command = parse_command(&["cyberfabric", "list", "local-modules", "--verbose"]);

    assert_eq!(
        command,
        CyberfabricCommand::List(cyberware_cli_core::list::ListArgs {
            command: cyberware_cli_core::list::ListCommand::LocalModules(
                cyberware_cli_core::list::LocalModulesArgs {
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
        "cyberfabric",
        "list",
        "system-modules",
        "--verbose",
        "--registry",
        "crates.io",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::List(cyberware_cli_core::list::ListArgs {
            command: cyberware_cli_core::list::ListCommand::SystemModules(
                cyberware_cli_core::list::SystemModulesArgs {
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
    let command = parse_command(&["cyberfabric", "list", "configs"]);

    assert_eq!(
        command,
        CyberfabricCommand::List(cyberware_cli_core::list::ListArgs {
            command: cyberware_cli_core::list::ListCommand::Configs(
                cyberware_cli_core::list::ConfigsArgs {
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}

#[test]
fn parses_list_apps_into_core_command() {
    let command = parse_command(&["cyberfabric", "list", "apps"]);

    assert_eq!(
        command,
        CyberfabricCommand::List(cyberware_cli_core::list::ListArgs {
            command: cyberware_cli_core::list::ListCommand::Apps(
                cyberware_cli_core::list::AppsArgs {
                    format: OutputFormat::Table,
                },
            ),
        })
    );
}
