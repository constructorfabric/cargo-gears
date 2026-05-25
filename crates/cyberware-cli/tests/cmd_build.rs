mod common;

use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::common::{BuildRunArgs, PathConfigArgs};
use std::path::PathBuf;

use common::parse_command;

#[test]
fn parses_build_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "build",
        "-c",
        "config.yml",
        "--otel",
        "--fips",
        "--release",
        "--clean",
        "--name",
        "demo-server",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Build(cyberware_cli_core::build::BuildArgs {
            build_run_args: BuildRunArgs {
                path_config: PathConfigArgs {
                    path: None,
                    config: PathBuf::from("config.yml"),
                },
                otel: true,
                fips: true,
                release: true,
                clean: true,
                name: Some("demo-server".to_owned()),
            },
        })
    );
}
