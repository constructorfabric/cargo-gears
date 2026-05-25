use clap::Parser;
use cyberware_cli::Cli;
use cyberware_cli_core::CyberfabricCommand;
use cyberware_cli_core::app_config::{DbConnConfig, DbEngineCfg, PoolCfg};
use cyberware_cli_core::common::{BuildRunArgs, OutputFormat, PathConfigArgs, Registry};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

use cyberware_cli_core::module_parser::test_utils::CWD_MUTEX;

fn parse_command(args: &[&str]) -> CyberfabricCommand {
    Cli::try_parse_from(args).expect("argv should parse").into()
}

#[test]
fn parses_lint_into_core_command() {
    let command = parse_command(&["cyberfabric", "lint", "--fmt", "--strict", "--clippy"]);

    assert_eq!(
        command,
        CyberfabricCommand::Lint(cyberware_cli_core::lint::LintArgs {
            all: false,
            path: None,
            fmt: true,
            clippy: true,
            strict: true,
            dylint: false,
        })
    );
}

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

#[test]
fn parses_tools_into_core_command() {
    let command = parse_command(&[
        "cyberfabric",
        "tools",
        "--install",
        "rustfmt,clippy",
        "--upgrade",
        "-y",
        "-v",
    ]);

    assert_eq!(
        command,
        CyberfabricCommand::Tools(cyberware_cli_core::tools::ToolsArgs {
            all: false,
            upgrade: true,
            install: Some(vec![
                cyberware_cli_core::tools::ToolName::Rustfmt,
                cyberware_cli_core::tools::ToolName::Clippy,
            ]),
            yolo: true,
            verbose: true,
        })
    );
}

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
                                config: PathBuf::from("config.yml"),
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
                                config: PathBuf::from("config.yml"),
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
                                        config: PathBuf::from("config.yml"),
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

#[test]
fn rejects_conflicting_tool_selection() {
    let result = Cli::try_parse_from(["cyberfabric", "tools", "--all", "--install", "rustfmt"]);
    let Err(error) = result else {
        panic!("conflicting tool selection should fail");
    };

    assert!(error.to_string().contains("cannot be used with"));
}

#[test]
fn path_parsing_changes_current_directory() -> anyhow::Result<()> {
    let _lock = CWD_MUTEX.lock().expect("cwd mutex should not be poisoned");
    let original_dir = std::env::current_dir()?;
    let temp_dir = tempfile::tempdir()?;

    let args = vec![
        OsString::from("cyberfabric"),
        OsString::from("lint"),
        OsString::from("-p"),
        temp_dir.path().as_os_str().to_owned(),
    ];
    let result = Cli::try_parse_from(args);
    let parsed_dir = std::env::current_dir()?;
    std::env::set_current_dir(&original_dir)?;

    result.expect("path should parse and change cwd");
    assert_eq!(parsed_dir, temp_dir.path().canonicalize()?);
    Ok(())
}
