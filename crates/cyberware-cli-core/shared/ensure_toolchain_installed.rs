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

#[cfg(feature = "dylint-rules")]
#[allow(dead_code)]
pub fn ensure_toolchain_components_installed(
    toolchain: &str,
    components: &[String],
) -> anyhow::Result<()> {
    if components.is_empty() {
        return Ok(());
    }

    let installed = Command::new("rustup")
        .args(["component", "list", "--toolchain", toolchain])
        .output()
        .with_context(|| format!("failed to list rustup components for `{toolchain}`"))?;

    if !installed.status.success() {
        bail!(
            "rustup component list failed for `{toolchain}`: {}",
            String::from_utf8_lossy(&installed.stderr)
        );
    }

    let installed = String::from_utf8(installed.stdout)
        .context("rustup component list returned non-UTF-8 output")?;
    for component in components {
        let installed_marker = format!("{component}-");
        let is_installed = installed.lines().any(|line| {
            line.contains("(installed)")
                && line
                    .split_whitespace()
                    .next()
                    .is_some_and(|name| name == component || name.starts_with(&installed_marker))
        });

        if is_installed {
            continue;
        }

        let install = Command::new("rustup")
            .args(["component", "add", component, "--toolchain", toolchain])
            .output()
            .with_context(|| {
                format!("failed to install rustup component `{component}` for `{toolchain}`")
            })?;

        if !install.status.success() {
            bail!(
                "rustup component install failed for `{component}` on `{toolchain}`: {}",
                String::from_utf8_lossy(&install.stderr)
            );
        }
    }

    Ok(())
}
