mod common;

use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::app_config::{DbConnConfig, DbEngineCfg, PoolCfg};
use cyberware_cli_core::common::PathConfigArgs;
use std::collections::BTreeMap;
use std::path::PathBuf;

use common::parse_command;

#[test]
fn parses_config_module_add_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "config",
        "mod",
        "add",
        "api-gateway",
        "-c",
        "config.yml",
        "--package",
        "cf-api-gateway",
        "--module-version",
        "1.0.0",
        "--default-features",
        "false",
        "-F",
        "grpc,otel",
        "--dep",
        "authz",
        "--dep",
        "tenant",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Config(cyberware_cli_core::config::ConfigArgs {
            command: cyberware_cli_core::config::ConfigCommand::Mod(
                cyberware_cli_core::config::modules::ModulesArgs {
                    command: cyberware_cli_core::config::modules::ModulesCommand::Add(
                        cyberware_cli_core::config::modules::add::AddArgs {
                            path_config: PathConfigArgs {
                                path: None,
                                config: Some(PathBuf::from("config.yml")),
                            },
                            module: "api-gateway".to_owned(),
                            package: Some("cf-api-gateway".to_owned()),
                            module_version: Some("1.0.0".to_owned()),
                            default_features: Some(false),
                            features: vec!["grpc".to_owned(), "otel".to_owned()],
                            deps: vec!["authz".to_owned(), "tenant".to_owned()],
                        },
                    ),
                },
            ),
        })
    );
}

#[test]
fn parses_config_db_add_into_core_command() {
    let mut params = BTreeMap::new();
    params.insert("connect_timeout".to_owned(), "10".to_owned());
    params.insert("sslmode".to_owned(), "require".to_owned());

    let command = parse_command(&[
        "cyberfabric",
        "config",
        "db",
        "add",
        "primary",
        "-c",
        "config.yml",
        "--engine",
        "postgres",
        "--host",
        "localhost",
        "--port",
        "5432",
        "--user",
        "cf",
        "--password",
        "secret",
        "--dbname",
        "app",
        "--params",
        "sslmode=require,connect_timeout=10",
        "--pool-max-conns",
        "20",
        "--pool-test-before-acquire",
        "true",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Config(cyberware_cli_core::config::ConfigArgs {
            command: cyberware_cli_core::config::ConfigCommand::Db(Box::new(
                cyberware_cli_core::config::db::DbArgs {
                    command: cyberware_cli_core::config::db::DbCommand::Add(
                        cyberware_cli_core::config::db::AddArgs {
                            path_config: PathConfigArgs {
                                path: None,
                                config: Some(PathBuf::from("config.yml")),
                            },
                            name: "primary".to_owned(),
                            conn: DbConnConfig {
                                engine: Some(DbEngineCfg::Postgres),
                                dsn: None,
                                host: Some("localhost".to_owned()),
                                port: Some(5432),
                                user: Some("cf".to_owned()),
                                password: Some("secret".to_owned()),
                                dbname: Some("app".to_owned()),
                                params: Some(params),
                                file: None,
                                path: None,
                                pool: Some(PoolCfg {
                                    max_conns: Some(20),
                                    min_conns: None,
                                    acquire_timeout: None,
                                    idle_timeout: None,
                                    max_lifetime: None,
                                    test_before_acquire: Some(true),
                                }),
                                server: None,
                            },
                        },
                    ),
                },
            )),
        })
    );
}

#[test]
fn parses_config_module_db_edit_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "config",
        "mod",
        "db",
        "edit",
        "api-gateway",
        "-c",
        "config.yml",
        "--server",
        "primary",
        "--pool-min-conns",
        "2",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Config(cyberware_cli_core::config::ConfigArgs {
            command: cyberware_cli_core::config::ConfigCommand::Mod(
                cyberware_cli_core::config::modules::ModulesArgs {
                    command: cyberware_cli_core::config::modules::ModulesCommand::Db(Box::new(
                        cyberware_cli_core::config::modules::db::ModuleDbArgs {
                            command: cyberware_cli_core::config::modules::db::ModuleDbCommand::Edit(
                                cyberware_cli_core::config::modules::db::EditArgs {
                                    path_config: PathConfigArgs {
                                        path: None,
                                        config: Some(PathBuf::from("config.yml")),
                                    },
                                    module: "api-gateway".to_owned(),
                                    conn: DbConnConfig {
                                        server: Some("primary".to_owned()),
                                        pool: Some(PoolCfg {
                                            min_conns: Some(2),
                                            ..PoolCfg::default()
                                        }),
                                        ..DbConnConfig::default()
                                    },
                                },
                            ),
                        },
                    )),
                },
            ),
        })
    );
}
