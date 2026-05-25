mod common;

use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::common::Registry;
use std::path::PathBuf;

use common::parse_command;

#[test]
fn parses_docs_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "docs",
        "-p",
        "workspace",
        "--registry",
        "crates.io",
        "-v",
        "--libs",
        "--version",
        "1.2.3",
        "--clean",
        "tokio::sync",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Docs(cyberware_cli_core::docs::DocsArgs {
            path: PathBuf::from("workspace"),
            registry: Registry::CratesIo,
            verbose: true,
            libs: true,
            version: Some(semver::Version::new(1, 2, 3)),
            clean: true,
            query: Some("tokio::sync".to_owned()),
        })
    );
}
