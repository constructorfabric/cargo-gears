use crate::common::cargo_cmd;
use anyhow::{Context, Result};

#[cfg(feature = "dylint-rules")]
use std::collections::BTreeSet;
#[cfg(feature = "dylint-rules")]
use std::fs;
#[cfg(feature = "dylint-rules")]
use std::io::ErrorKind;
#[cfg(feature = "dylint-rules")]
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(feature = "dylint-rules")]
mod ensure_toolchain_installed_shared {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shared/ensure_toolchain_installed.rs"
    ));
}

#[cfg(feature = "dylint-rules")]
use ensure_toolchain_installed_shared::ensure_toolchain_installed;

#[derive(Debug, Eq, PartialEq)]
pub struct LintParams {
    /// Resolved workspace root path.
    pub workspace_root: PathBuf,
    /// Check whether the workspace is formatted with `cargo fmt`.
    pub fmt: bool,
    /// Run recommended clippy rules. Follows Cargo.toml exceptions if present.
    pub clippy: bool,
    /// Strict mode. Throws an error if any lint rule is triggered.
    pub strict: bool,
    /// Run extra lint rules made for gears modules.
    pub dylint: bool,
    /// Lint names to skip when running dylint.
    pub dylint_skip: Vec<String>,
}

#[cfg(feature = "dylint-rules")]
include!(concat!(env!("OUT_DIR"), "/generated_libs.rs"));

impl LintParams {
    pub fn run(&self) -> Result<()> {
        if self.fmt {
            run_fmt(&self.workspace_root)?;
        }

        if self.clippy {
            run_clippy(&self.workspace_root, self.strict)?;
        }

        if self.dylint {
            run_dylint(&self.workspace_root, &self.dylint_skip)?;
        }

        Ok(())
    }
}

fn run_fmt(workspace_path: &Path) -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.args(["fmt", "--check", "--all"]);
    cmd.current_dir(workspace_path);

    let status = cmd.status().context("failed to run `cargo fmt --check`")?;
    if !status.success() {
        anyhow::bail!("`cargo fmt --check` failed with exit status {status}");
    }

    Ok(())
}

fn run_clippy(workspace_path: &Path, strict: bool) -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.args(["clippy", "--workspace", "--all-targets"]);
    cmd.current_dir(workspace_path);

    // TODO Analyse the manifest feature-set policy and lint those combinations.

    if strict {
        cmd.arg("--").arg("-D").arg("warnings");
    }

    let status = cmd.status().context("failed to run `cargo clippy`")?;
    if !status.success() {
        anyhow::bail!("`cargo clippy` failed with exit status {status}");
    }

    Ok(())
}

#[cfg(feature = "dylint-rules")]
fn embedded_toolchains() -> Result<BTreeSet<String>> {
    LIBS.iter()
        .map(|(filename, _)| {
            let (_, toolchain_and_ext) = filename
                .rsplit_once('@')
                .with_context(|| format!("missing toolchain marker in `{filename}`"))?;
            let (toolchain, _) = toolchain_and_ext
                .rsplit_once('.')
                .with_context(|| format!("missing library extension in `{filename}`"))?;
            Ok(toolchain.to_owned())
        })
        .collect()
}

#[cfg(feature = "dylint-rules")]
fn run_dylint(workspace_path: &Path, skipped_lints: &[String]) -> Result<()> {
    for toolchain in embedded_toolchains()? {
        ensure_toolchain_installed(&toolchain)?;
        clear_dylint_rustc_info_cache(workspace_path, &toolchain)?;
    }

    // Write every embedded dylib to a per-run temp directory so dylint can
    // dlopen them. The temp dir (and its contents) is removed when `tmp_dir`
    // drops at the end of this function, which is safe because `dylint::run`
    // is synchronous and has already finished using the files by then.
    let tmp_dir = tempfile::tempdir().context("could not create temp dir for dylibs")?;

    let lib_paths: Vec<String> = LIBS
        .iter()
        .map(|(filename, bytes)| {
            let dest = tmp_dir.path().join(filename);
            let mut f = std::fs::File::create(&dest)
                .with_context(|| format!("could not create {filename} in temp dir"))?;
            f.write_all(bytes)
                .with_context(|| format!("could not write {filename} to temp dir"))?;
            Ok(dest.to_string_lossy().into_owned())
        })
        .collect::<Result<_>>()?;

    let opts = dylint::opts::Dylint {
        operation: dylint::opts::Operation::Check(dylint::opts::Check {
            lib_sel: dylint::opts::LibrarySelection {
                // Point directly at the extracted, versioned dylib files.
                // dylint parses the toolchain from each filename so no further
                // discovery or building is necessary.
                lib_paths,
                // Check all packages in the workspace rooted at `workspace_path`.
                // Pointing Dylint at the workspace manifest avoids depending on
                // the process CWD.
                manifest_path: Some(
                    workspace_path
                        .join("Cargo.toml")
                        .to_string_lossy()
                        .into_owned(),
                ),
                ..Default::default()
            },
            // Lint the whole workspace, not just the root crate.
            workspace: true,
            args: dylint_cargo_check_args(skipped_lints)?,
            ..Default::default()
        }),
        ..Default::default()
    };

    dylint::run(&opts)
}

#[cfg(feature = "dylint-rules")]
fn dylint_cargo_check_args(skipped_lints: &[String]) -> Result<Vec<String>> {
    if skipped_lints.is_empty() {
        return Ok(Vec::new());
    }

    let rustflags = skipped_lints
        .iter()
        .flat_map(|lint| ["-A".to_owned(), lint.clone()])
        .collect::<Vec<_>>();
    let rustflags = serde_json::to_string(&rustflags).context("failed to encode dylint skips")?;

    Ok(vec![
        "--config".to_owned(),
        format!("build.rustflags={rustflags}"),
    ])
}

#[cfg(feature = "dylint-rules")]
fn clear_dylint_rustc_info_cache(workspace_path: &Path, toolchain: &str) -> Result<()> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(workspace_path.join("Cargo.toml"))
        .no_deps()
        .exec()
        .context("failed to resolve workspace metadata for dylint target dir")?;

    let rustc_info = metadata
        .target_directory
        .as_std_path()
        .join("dylint/target")
        .join(toolchain)
        .join(".rustc_info.json");

    match fs::remove_file(&rustc_info) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| {
            format!(
                "failed to clear stale dylint rustc info cache at {}",
                rustc_info.display()
            )
        }),
    }
}

#[cfg(not(feature = "dylint-rules"))]
fn run_dylint(_workspace_path: &Path, _skipped_lints: &[String]) -> Result<()> {
    anyhow::bail!("dylint-rules feature not enabled")
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "dylint-rules")]
    #[test]
    fn dylint_skip_list_is_converted_to_cargo_rustflags_config() {
        let args = super::dylint_cargo_check_args(&[
            "de0301_no_infra_in_domain".to_owned(),
            "de1302_error_from_to_string".to_owned(),
        ])
        .expect("skip args should encode");

        assert_eq!(
            args,
            vec![
                "--config".to_owned(),
                "build.rustflags=[\"-A\",\"de0301_no_infra_in_domain\",\"-A\",\"de1302_error_from_to_string\"]"
                    .to_owned(),
            ]
        );
    }
}
