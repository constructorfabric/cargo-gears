mod common;

use cargo_gears_core::manifest::TestRunner;
use common::parse_cli;

#[test]
fn parses_test_flags() {
    let cli = parse_cli(&[
        "gears",
        "test",
        "--manifest",
        "Gears.dev.toml",
        "--app",
        "app1",
        "--env",
        "dev",
        "--runner",
        "nextest",
        "--module",
        "module-a",
        "--coverage",
    ]);

    let cargo_gears::Commands::Test(args) = cli.command() else {
        panic!("expected test command");
    };

    assert_eq!(
        args.manifest.manifest_path.manifest.to_str(),
        Some("Gears.dev.toml")
    );
    assert_eq!(args.manifest.app.as_deref(), Some("app1"));
    assert_eq!(args.manifest.env.as_deref(), Some("dev"));
    assert_eq!(args.runner, Some(TestRunner::Nextest));
    assert_eq!(args.module.as_deref(), Some("module-a"));
    assert!(args.coverage);
}

#[test]
fn parses_test_defaults() {
    let cli = parse_cli(&["gears", "test"]);

    let cargo_gears::Commands::Test(args) = cli.command() else {
        panic!("expected test command");
    };

    assert_eq!(
        args.manifest.manifest_path.manifest.to_str(),
        Some("Gears.toml")
    );
    assert_eq!(args.manifest.app, None);
    assert_eq!(args.manifest.env, None);
    assert_eq!(args.runner, None);
    assert_eq!(args.module, None);
    assert!(!args.coverage);
}
