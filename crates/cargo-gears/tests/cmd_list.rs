mod common;

use cargo_gears::Cli;
use cargo_gears_core::GearsCommand;
use cargo_gears_core::common::{OutputFormat, Registry};
use clap::Parser;

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
                    filter: None,
                    scope_dirs: Vec::new(),
                    include_rdeps: false,
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
                    filter: None,
                    scope_dirs: Vec::new(),
                    include_rdeps: false,
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
                    filter: None,
                    scope_dirs: Vec::new(),
                    include_rdeps: false,
                },
            ),
        })
    );
}

#[test]
fn parses_list_templates_into_core_command() {
    let command = parse_command(&["gears", "ls", "templates"]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Templates(
                cargo_gears_core::list::TemplatesParams { path: None },
            ),
        })
    );
}

#[test]
fn parses_list_gears_with_filter() {
    let command = parse_command(&["gears", "ls", "gears", "--local", "--filter", "api-.*"]);

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
                    filter: Some("api-.*".to_owned()),
                    scope_dirs: Vec::new(),
                    include_rdeps: false,
                },
            ),
        })
    );
}

#[test]
fn parses_list_gears_with_scope_dirs_and_include_rdeps() {
    let command = parse_command(&[
        "gears",
        "ls",
        "gears",
        "--local",
        "--scope-dirs",
        "gears/api,gears/db",
        "--include-rdeps",
        "--format",
        "cargo-flags",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Gears(
                cargo_gears_core::list::GearsParams {
                    path: None,
                    verbose: false,
                    output: cargo_gears_core::list::GearsOutput::local(),
                    registry: Registry::CratesIo,
                    format: OutputFormat::CargoFlags,
                    filter: None,
                    scope_dirs: vec!["gears/api".to_owned(), "gears/db".to_owned()],
                    include_rdeps: true,
                },
            ),
        })
    );
}

#[test]
fn parses_list_features_into_core_command() {
    let command = parse_command(&[
        "gears",
        "ls",
        "features",
        "--manifest",
        "Cargo.toml",
        "-f",
        "json",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Features(
                cargo_gears_core::list::FeaturesParams {
                    manifest: std::path::PathBuf::from("Cargo.toml"),
                    format: OutputFormat::Json,
                },
            ),
        })
    );
}

#[test]
fn parses_list_deps_into_core_command() {
    let command = parse_command(&[
        "gears",
        "ls",
        "deps",
        "--manifest",
        "Cargo.toml",
        "--non-optional",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Deps(
                cargo_gears_core::list::DepsParams {
                    manifest: std::path::PathBuf::from("Cargo.toml"),
                    non_optional: true,
                    format: OutputFormat::List,
                },
            ),
        })
    );
}

#[test]
fn parses_list_packages_into_core_command() {
    let command = parse_command(&[
        "gears",
        "ls",
        "packages",
        "--scope-dirs",
        "gears",
        "--filter",
        "cf-.*",
        "--include-rdeps",
        "-f",
        "cargo-flags",
    ]);

    assert_eq!(
        command,
        GearsCommand::List(cargo_gears_core::list::ListParams {
            command: cargo_gears_core::list::ListCommand::Packages(
                cargo_gears_core::list::PackagesParams {
                    path: None,
                    scope_dirs: vec!["gears".to_owned()],
                    filter: Some("cf-.*".to_owned()),
                    include_rdeps: true,
                    format: OutputFormat::CargoFlags,
                },
            ),
        })
    );
}

#[test]
fn parses_check_version_top_level_flag() {
    let cli = Cli::try_parse_from(["gears", "--check-version", ">=0.0.1"]).expect("should parse");
    // --check-version is handled in Cli::run(), not via TryFrom, so TryFrom should fail
    // because there's no subcommand
    let result = GearsCommand::try_from(cli);
    assert!(result.is_err());
}
