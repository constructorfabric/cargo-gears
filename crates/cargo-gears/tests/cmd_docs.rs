mod common;

use cargo_gears_core::GearsCommand;
use cargo_gears_core::common::Registry;
use std::path::PathBuf;

use common::parse_command;

#[test]
fn parses_src_into_core_command() {
    let command = parse_command(&[
        "gears",
        "src",
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
        GearsCommand::Src(cargo_gears_core::source::SourceParams {
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
