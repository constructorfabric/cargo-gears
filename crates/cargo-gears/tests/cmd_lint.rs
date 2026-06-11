use cargo_gears::Cli;
use clap::Parser;
use std::ffi::OsString;

#[test]
fn path_parsing_does_not_change_current_directory() -> anyhow::Result<()> {
    let original_dir = std::env::current_dir()?;
    let temp_dir = tempfile::tempdir()?;

    let args = vec![
        OsString::from("gears"),
        OsString::from("lint"),
        OsString::from("-p"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--app"),
        OsString::from("app1"),
        OsString::from("--env"),
        OsString::from("dev"),
    ];
    Cli::try_parse_from(args).expect("path should parse successfully");

    // CWD must remain unchanged after parsing
    assert_eq!(std::env::current_dir()?, original_dir);
    Ok(())
}
