mod common;

use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::common::{BuildRunArgs, PathConfigArgs};
use std::path::PathBuf;

use common::parse_command;

#[test]
fn parses_run_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "run",
        "--watch",
        "-c",
        "config.yml",
        "--release",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Run(cyberware_cli_core::run::RunArgs {
            watch: true,
            br_args: BuildRunArgs {
                path_config: PathConfigArgs {
                    path: None,
                    config: PathBuf::from("config.yml"),
                },
                otel: false,
                fips: false,
                release: true,
                clean: false,
                name: None,
            },
        })
    );
}
