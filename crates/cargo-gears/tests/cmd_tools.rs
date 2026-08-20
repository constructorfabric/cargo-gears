mod common;

use cargo_gears::Cli;
use cargo_gears_core::GearsCommand;
use clap::Parser;
use std::process::Command;

fn cargo_gears_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-gears"))
}

#[test]
fn try_from_returns_error_for_tools_command() {
    use std::convert::TryFrom;

    let cli = Cli::try_parse_from(["gears", "tools", "--install", "rustfmt,clippy", "--upgrade"])
        .expect("should parse");
    let result = GearsCommand::try_from(cli);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "command should be dispatched directly in Cli::run()"
    );
}

#[test]
fn parses_tools_check_version() {
    let cli = Cli::try_parse_from(["gears", "tools", "check-version", "cargo-deny", ">=0.20.0"])
        .expect("should parse");
    // check-version is handled in ToolsArgs::run(), not via TryFrom
    let result = GearsCommand::try_from(cli);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "command should be dispatched directly in Cli::run()"
    );
}

#[test]
fn rejects_conflicting_tool_selection() {
    let result = Cli::try_parse_from(["gears", "tools", "--all", "--install", "rustfmt"]);
    let Err(error) = result else {
        panic!("conflicting tool selection should fail");
    };

    assert!(error.to_string().contains("cannot be used with"));
}

// --- Integration tests: tools check-version (subprocess) ---

#[test]
fn check_version_succeeds_for_installed_tool() {
    // `cargo --version` is always available during `cargo test`
    let output = cargo_gears_bin()
        .args(["gears", "tools", "check-version", "cargo", ">=1.0.0"])
        .output()
        .expect("should spawn");
    assert!(
        output.status.success(),
        "check-version should succeed for cargo >=1.0.0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        semver::Version::parse(stdout.trim()).is_ok(),
        "stdout should contain a valid semver version, got: {stdout}"
    );
}

#[test]
fn check_version_fails_for_unsatisfied_requirement() {
    let output = cargo_gears_bin()
        .args(["gears", "tools", "check-version", "cargo", ">=9999.0.0"])
        .output()
        .expect("should spawn");
    assert!(
        !output.status.success(),
        "check-version should fail for cargo >=9999.0.0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not satisfy"),
        "stderr should indicate version mismatch, got: {stderr}"
    );
}

#[test]
fn check_version_fails_for_missing_tool() {
    let output = cargo_gears_bin()
        .args([
            "gears",
            "tools",
            "check-version",
            "nonexistent-tool-xyz-12345",
            ">=1.0.0",
        ])
        .output()
        .expect("should spawn");
    assert!(
        !output.status.success(),
        "check-version should fail for missing tool"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("is not installed"),
        "stderr should say tool is not installed, got: {stderr}"
    );
}

#[test]
fn check_version_parses_version_from_stderr() {
    // Create a script that writes version to stderr and nothing to stdout
    let temp = tempfile::TempDir::new().expect("temp dir");
    let script = temp.path().join("fake-tool");
    std::fs::write(&script, "#!/bin/sh\necho 'fake-tool 2.5.0' >&2\n").expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    let output = cargo_gears_bin()
        .args([
            "gears",
            "tools",
            "check-version",
            script.to_str().expect("path"),
            ">=2.0.0",
        ])
        .output()
        .expect("should spawn");
    assert!(
        output.status.success(),
        "check-version should parse version from stderr, stderr: {}, stdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim() == "2.5.0",
        "should print parsed version, got: {stdout}"
    );
}
