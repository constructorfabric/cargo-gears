mod common;

use cyberware_cli_core::CyberfabricCommand;
use std::path::PathBuf;

use common::parse_command;

#[test]
fn parses_module_template_enum_into_core_command() {
    let command = parse_command(&["cyberfabric", "mod", "add", "api-db-handler"]);

    assert_eq!(
        command,
        CyberfabricCommand::Mod(cyberware_cli_core::r#mod::ModArgs {
            command: cyberware_cli_core::r#mod::ModCommand::Add(
                cyberware_cli_core::r#mod::add::AddArgs {
                    name: cyberware_cli_core::r#mod::add::ModuleTemplateName::ApiDbHandler,
                    path: PathBuf::from("."),
                    verbose: false,
                    local_path: None,
                    git: Some("https://github.com/cyberfabric/cf-template-rust".to_owned()),
                    subfolder: "Modules".to_owned(),
                    branch: Some("main".to_owned()),
                },
            ),
        })
    );
}
