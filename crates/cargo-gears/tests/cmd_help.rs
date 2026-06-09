mod common;

use cargo_gears_core::GearsCommand;
use cargo_gears_core::help::{
    HelpCommand, HelpParams, SchemaParams, SchemaTarget, Topic, TopicParams,
};
use common::{assert_parse_error, parse_command};
use std::path::PathBuf;

#[test]
fn parses_help_schema_manifest() {
    let command = parse_command(&["gears", "help", "schema", "manifest"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Schema(SchemaParams {
                target: SchemaTarget::Manifest,
                section: None,
            }),
        })
    );
}

#[test]
fn parses_help_schema_config_with_section() {
    let command = parse_command(&["gears", "help", "schema", "config", "--section", "database"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Schema(SchemaParams {
                target: SchemaTarget::Config,
                section: Some("database".to_owned()),
            }),
        })
    );
}

#[test]
fn parses_help_schema_module() {
    let command = parse_command(&["gears", "help", "schema", "module"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Schema(SchemaParams {
                target: SchemaTarget::Module,
                section: None,
            }),
        })
    );
}

#[test]
fn parses_help_src_with_query() {
    let command = parse_command(&["gears", "help", "src", "tokio::sync"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Src(cargo_gears_core::source::SourceParams {
                path: PathBuf::from("."),
                registry: cargo_gears_core::common::Registry::CratesIo,
                verbose: false,
                libs: false,
                version: None,
                clean: false,
                query: Some("tokio::sync".to_owned()),
            }),
        })
    );
}

#[test]
fn parses_help_src_with_all_flags() {
    let command = parse_command(&[
        "gears",
        "help",
        "src",
        "-p",
        "/tmp/ws",
        "-v",
        "--libs",
        "--version",
        "1.0.0",
        "--clean",
        "cf-modkit",
    ]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Src(cargo_gears_core::source::SourceParams {
                path: PathBuf::from("/tmp/ws"),
                registry: cargo_gears_core::common::Registry::CratesIo,
                verbose: true,
                libs: true,
                version: Some(semver::Version::new(1, 0, 0)),
                clean: true,
                query: Some("cf-modkit".to_owned()),
            }),
        })
    );
}

#[test]
fn parses_help_topic_architecture() {
    let command = parse_command(&["gears", "help", "topic", "architecture"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::Architecture,
            }),
        })
    );
}

#[test]
fn parses_help_topic_cli() {
    let command = parse_command(&["gears", "help", "topic", "cli"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams { topic: Topic::Cli }),
        })
    );
}

#[test]
fn parses_help_topic_clienthub() {
    let command = parse_command(&["gears", "help", "topic", "clienthub"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::ClientHub,
            }),
        })
    );
}

#[test]
fn parses_help_topic_database() {
    let command = parse_command(&["gears", "help", "topic", "database"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::Database,
            }),
        })
    );
}

#[test]
fn parses_help_topic_errors() {
    let command = parse_command(&["gears", "help", "topic", "errors"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::Errors,
            }),
        })
    );
}

#[test]
fn parses_help_topic_fips() {
    let command = parse_command(&["gears", "help", "topic", "fips"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams { topic: Topic::Fips }),
        })
    );
}

#[test]
fn parses_help_topic_gears_catalog() {
    let command = parse_command(&["gears", "help", "topic", "gears-catalog"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::GearsCatalog,
            }),
        })
    );
}

#[test]
fn parses_help_topic_generated_server() {
    let command = parse_command(&["gears", "help", "topic", "generated-server"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::GeneratedServer,
            }),
        })
    );
}

#[test]
fn parses_help_topic_lifecycle() {
    let command = parse_command(&["gears", "help", "topic", "lifecycle"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::Lifecycle,
            }),
        })
    );
}

#[test]
fn parses_help_topic_manifest() {
    let command = parse_command(&["gears", "help", "topic", "manifest"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::Manifest,
            }),
        })
    );
}

#[test]
fn parses_help_topic_module_layout() {
    let command = parse_command(&["gears", "help", "topic", "module-layout"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::ModuleLayout,
            }),
        })
    );
}

#[test]
fn parses_help_topic_module_refs() {
    let command = parse_command(&["gears", "help", "topic", "module-refs"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::ModuleRefs,
            }),
        })
    );
}

#[test]
fn parses_help_topic_otel() {
    let command = parse_command(&["gears", "help", "topic", "otel"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams { topic: Topic::Otel }),
        })
    );
}

#[test]
fn parses_help_topic_rest_api() {
    let command = parse_command(&["gears", "help", "topic", "rest-api"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::RestApi,
            }),
        })
    );
}

#[test]
fn parses_help_topic_security() {
    let command = parse_command(&["gears", "help", "topic", "security"]);

    assert_eq!(
        command,
        GearsCommand::Help(HelpParams {
            command: HelpCommand::Topic(TopicParams {
                topic: Topic::Security,
            }),
        })
    );
}

#[test]
fn help_schema_rejects_unknown_target() {
    assert_parse_error(
        &["gears", "help", "schema", "bogus"],
        clap::error::ErrorKind::InvalidValue,
    );
}

#[test]
fn help_topic_rejects_unknown_topic() {
    assert_parse_error(
        &["gears", "help", "topic", "bogus"],
        clap::error::ErrorKind::InvalidValue,
    );
}
