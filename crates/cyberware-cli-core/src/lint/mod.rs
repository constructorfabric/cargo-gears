use crate::common::cargo_cmd;
use anyhow::{Context, Result};

#[cfg(feature = "dylint-rules")]
use std::collections::BTreeSet;
#[cfg(feature = "dylint-rules")]
use std::io::Write;
use std::path::PathBuf;

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
pub struct LintArgs {
    /// Run all available lint rules
    pub all: bool,
    /// Path to the module workspace root
    pub path: Option<PathBuf>,
    /// Check whether the workspace is formatted with `cargo fmt`.
    pub fmt: bool,
    /// Run recommended clippy rules. Follows Cargo.toml exceptions if present.
    pub clippy: bool,
    /// Strict mode. Throws an error if any lint rule is triggered.
    pub strict: bool,
    /// Run extra lint rules made for cyberfabric modules.
    pub dylint: bool,
}

#[cfg(feature = "dylint-rules")]
include!(concat!(env!("OUT_DIR"), "/generated_libs.rs"));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EffectiveLintSelection {
    all: bool,
    fmt: bool,
    clippy: bool,
    dylint: bool,
}

impl LintArgs {
    const fn selection(&self) -> EffectiveLintSelection {
        let all = self.all || (!self.fmt && !self.clippy && !self.dylint);
        EffectiveLintSelection {
            all,
            fmt: self.fmt,
            clippy: self.clippy || all,
            dylint: self.dylint || (all && cfg!(feature = "dylint-rules")),
        }
    }

    fn validate(&self) -> Result<EffectiveLintSelection> {
        let selection = self.selection();
        if self.strict && !selection.clippy {
            anyhow::bail!("`--strict` requires `--clippy` or `--all`");
        }
        Ok(selection)
    }

    pub fn run(&self) -> Result<()> {
        crate::common::with_current_dir_for_optional_path(self.path.as_deref(), || {
            let selection = self.validate()?;

            if selection.fmt {
                run_fmt()?;
            }

            if selection.clippy {
                run_clippy(self.strict)?;
            }

            if selection.dylint {
                run_dylint()?;
            }

            Ok(())
        })
    }
}

fn run_fmt() -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.args(["fmt", "--check", "--all"]);

    let status = cmd.status().context("failed to run `cargo fmt --check`")?;
    if !status.success() {
        anyhow::bail!("`cargo fmt --check` failed with exit status {status}");
    }

    Ok(())
}

fn run_clippy(strict: bool) -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.args(["clippy", "--workspace", "--all-targets", "--all-features"]);

    // TODO Analyse the features that each crate has and try to test them against the feature set.

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
fn run_dylint() -> Result<()> {
    for toolchain in embedded_toolchains()? {
        ensure_toolchain_installed(&toolchain)?;
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
        // Check all packages in the workspace found in the current working
        // directory.  No manifest_path → dylint resolves the workspace from
        // the CWD, which is exactly what we want when the tool is invoked
        // inside a project.
        operation: dylint::opts::Operation::Check(dylint::opts::Check {
            lib_sel: dylint::opts::LibrarySelection {
                // Point directly at the extracted, versioned dylib files.
                // dylint parses the toolchain from each filename so no further
                // discovery or building is necessary.
                lib_paths,
                ..Default::default()
            },
            // Lint the whole workspace, not just the root crate.
            workspace: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    dylint::run(&opts)
}

#[cfg(not(feature = "dylint-rules"))]
fn run_dylint() -> Result<()> {
    anyhow::bail!("dylint-rules feature not enabled")
}

#[cfg(test)]
mod tests {
    use super::LintArgs;

    #[allow(clippy::fn_params_excessive_bools)]
    fn lint_args(all: bool, fmt: bool, clippy: bool, strict: bool, dylint: bool) -> LintArgs {
        LintArgs {
            all,
            path: None,
            fmt,
            clippy,
            strict,
            dylint,
        }
    }

    #[test]
    fn defaults_to_all_lints() {
        let args = lint_args(false, false, false, false, false);

        let selection = args.selection();

        assert!(selection.all);
        assert!(!selection.fmt);
        assert!(selection.clippy);
        #[cfg(feature = "dylint-rules")]
        assert!(selection.dylint);
        #[cfg(not(feature = "dylint-rules"))]
        assert!(!selection.dylint);
    }

    #[test]
    fn explicit_lint_selection_disables_default_all() {
        let args = lint_args(false, false, false, false, true);

        let selection = args.selection();

        assert!(!selection.all);
        assert!(!selection.fmt);
        assert!(!selection.clippy);
        assert!(selection.dylint);
    }

    #[test]
    fn fmt_selection_is_explicit() {
        let args = lint_args(false, true, false, false, false);

        let selection = args.selection();

        assert!(!selection.all);
        assert!(selection.fmt);
        assert!(!selection.clippy);
        assert!(!selection.dylint);
    }

    #[test]
    fn strict_with_clippy_is_accepted() {
        let args = lint_args(false, false, true, true, false);

        args.validate()
            .expect("strict with clippy should be accepted");
    }

    #[test]
    fn strict_with_all_is_accepted() {
        let args = lint_args(true, false, false, true, false);

        args.validate().expect("strict with all should be accepted");
    }

    #[test]
    fn strict_requires_clippy_or_all() {
        let args = lint_args(false, false, false, true, true);

        let error = args.validate().expect_err("strict should be rejected");

        assert_eq!(
            error.to_string(),
            "`--strict` requires `--clippy` or `--all`"
        );
    }
}
