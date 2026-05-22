#[cfg(feature = "dylint-rules")]
use anyhow::{Context, bail};
#[cfg(feature = "dylint-rules")]
use std::process::Command;

#[cfg(feature = "dylint-rules")]
pub fn ensure_toolchain_installed(toolchain: &str) -> anyhow::Result<()> {
    let installed = Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .context("failed to list installed rustup toolchains")?;

    if !installed.status.success() {
        bail!(
            "rustup toolchain list failed: {}",
            String::from_utf8_lossy(&installed.stderr)
        );
    }

    let installed = String::from_utf8(installed.stdout)
        .context("rustup toolchain list returned non-UTF-8 output")?;
    let installed_prefix = format!("{toolchain}-");
    if installed
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .any(|installed| installed == toolchain || installed.starts_with(&installed_prefix))
    {
        return Ok(());
    }

    let install = Command::new("rustup")
        .args(["toolchain", "install", toolchain, "--profile", "minimal"])
        .output()
        .with_context(|| format!("failed to install rustup toolchain `{toolchain}`"))?;

    if !install.status.success() {
        bail!(
            "rustup toolchain install failed for `{toolchain}`: {}",
            String::from_utf8_lossy(&install.stderr)
        );
    }

    Ok(())
}
