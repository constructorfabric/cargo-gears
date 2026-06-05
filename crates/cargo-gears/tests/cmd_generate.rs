mod common;

use cargo_gears_core::GearsCommand;
use cargo_gears_core::generate::{GenerateCommand, GenerateParams};
use common::parse_command;
use std::path::PathBuf;

#[test]
fn parses_generate_workspace_into_core_command() {
    let command = parse_command(&[
        "gears",
        "generate",
        "workspace",
        "/tmp/cf-demo",
        "--project",
        "cf-demo",
    ]);

    assert_eq!(
        command,
        GearsCommand::Generate(GenerateParams {
            command: GenerateCommand::Workspace(
                cargo_gears_core::generate::workspace::WorkspaceParams {
                    path: PathBuf::from("/tmp/cf-demo"),
                    template: "default".to_owned(),
                    name: Some("cf-demo".to_owned()),
                    verbose: false,
                    local_path: None,
                    git: None,
                    subfolder: None,
                    branch: None,
                    r#override: false,
                }
            ),
        })
    );
}

#[test]
fn parses_new_alias_into_core_command() {
    let command = parse_command(&["gears", "new", "/tmp/cf-demo"]);

    assert_eq!(
        command,
        GearsCommand::Generate(GenerateParams {
            command: GenerateCommand::Workspace(
                cargo_gears_core::generate::workspace::WorkspaceParams {
                    path: PathBuf::from("/tmp/cf-demo"),
                    template: "default".to_owned(),
                    name: None,
                    verbose: false,
                    local_path: None,
                    git: None,
                    subfolder: None,
                    branch: None,
                    r#override: false,
                }
            ),
        })
    );
}

#[test]
fn parses_generate_workspace_with_custom_template() {
    let command = parse_command(&[
        "gears",
        "generate",
        "workspace",
        "/tmp/cf-demo",
        "--template",
        "custom",
    ]);

    assert_eq!(
        command,
        GearsCommand::Generate(GenerateParams {
            command: GenerateCommand::Workspace(
                cargo_gears_core::generate::workspace::WorkspaceParams {
                    path: PathBuf::from("/tmp/cf-demo"),
                    template: "custom".to_owned(),
                    name: None,
                    verbose: false,
                    local_path: None,
                    git: None,
                    subfolder: None,
                    branch: None,
                    r#override: false,
                }
            ),
        })
    );
}

#[test]
fn parses_generate_module_into_core_command() {
    let command = parse_command(&[
        "gears",
        "generate",
        "module",
        "--template",
        "api-db-handler",
    ]);

    assert_eq!(
        command,
        GearsCommand::Generate(GenerateParams {
            command: GenerateCommand::Module(cargo_gears_core::generate::module::ModuleParams {
                template: "api-db-handler".to_owned(),
                name: None,
                path: PathBuf::from("."),
                verbose: false,
                local_path: None,
                git: None,
                subfolder: None,
                branch: None,
            }),
        })
    );
}

#[test]
fn parses_generate_module_with_name() {
    let command = parse_command(&[
        "gears",
        "generate",
        "module",
        "--template",
        "background-worker",
        "--name",
        "jobs",
    ]);

    assert_eq!(
        command,
        GearsCommand::Generate(GenerateParams {
            command: GenerateCommand::Module(cargo_gears_core::generate::module::ModuleParams {
                template: "background-worker".to_owned(),
                name: Some("jobs".to_owned()),
                path: PathBuf::from("."),
                verbose: false,
                local_path: None,
                git: None,
                subfolder: None,
                branch: None,
            }),
        })
    );
}

#[test]
fn parses_generate_config_into_core_command() {
    let command = parse_command(&[
        "gears",
        "generate",
        "config",
        "--template",
        "dev",
        "--app",
        "myapp",
        "--env",
        "staging",
    ]);

    assert_eq!(
        command,
        GearsCommand::Generate(GenerateParams {
            command: GenerateCommand::Config(
                cargo_gears_core::generate::config::GenerateConfigParams {
                    template: "dev".to_owned(),
                    app: Some("myapp".to_owned()),
                    env: Some("staging".to_owned()),
                    name: None,
                    path: PathBuf::from("."),
                }
            ),
        })
    );
}
