use crate::common::cargo_cmd;
use crate::manifest::{LintPolicy, ManifestSelection};
use anyhow::{Context, Result};

#[cfg(feature = "dylint-rules")]
use std::collections::BTreeSet;
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
    /// Run all available lint rules
    pub all: bool,
    /// Path to the module workspace root
    pub path: Option<PathBuf>,
    pub manifest: ManifestSelection,
    /// Check whether the workspace is formatted with `cargo fmt`.
    pub fmt: bool,
    /// Run recommended clippy rules. Follows Cargo.toml exceptions if present.
    pub clippy: bool,
    /// Strict mode. Throws an error if any lint rule is triggered.
    pub strict: bool,
    /// Run extra lint rules made for gears modules.
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

impl LintParams {
    const fn has_explicit_selection(&self) -> bool {
        self.all || self.fmt || self.clippy || self.dylint
    }

    fn selection(&self, policy: &LintPolicy) -> EffectiveLintSelection {
        if !self.has_explicit_selection() {
            return EffectiveLintSelection {
                all: false,
                fmt: policy.fmt,
                clippy: policy.clippy,
                dylint: policy.dylint.as_ref().is_some_and(|dylint| dylint.enabled),
            };
        }

        let all = self.all;
        EffectiveLintSelection {
            all,
            fmt: self.fmt || all,
            clippy: self.clippy || all,
            dylint: self.dylint || (all && cfg!(feature = "dylint-rules")),
        }
    }

    fn validate(&self, policy: &LintPolicy) -> Result<EffectiveLintSelection> {
        let selection = self.selection(policy);
        if self.strict && !selection.clippy {
            anyhow::bail!("`--strict` requires `--clippy` or `--all`");
        }
        Ok(selection)
    }

    pub fn run(&self) -> Result<()> {
        let workspace_path = crate::common::resolve_workspace_path(self.path.as_deref())?;
        let resolved = self.manifest.resolve(&workspace_path)?;
        let selection = self.validate(&resolved.lint)?;

        if selection.fmt {
            run_fmt(&resolved.workspace_root)?;
        }

        if selection.clippy {
            run_clippy(&resolved.workspace_root, self.strict)?;
        }

        if selection.dylint {
            run_dylint(&resolved.workspace_root)?;
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
fn run_dylint(workspace_path: &Path) -> Result<()> {
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
            ..Default::default()
        }),
        ..Default::default()
    };

    dylint::run(&opts)
}

#[cfg(not(feature = "dylint-rules"))]
fn run_dylint(_workspace_path: &Path) -> Result<()> {
    anyhow::bail!("dylint-rules feature not enabled")
}

#[cfg(test)]
mod tests {
    use super::LintParams;
    use crate::manifest::{Dylint, LintPolicy, ManifestSelection};
    use std::path::PathBuf;

    #[allow(clippy::fn_params_excessive_bools)]
    fn lint_args(all: bool, fmt: bool, clippy: bool, strict: bool, dylint: bool) -> LintParams {
        LintParams {
            all,
            path: None,
            manifest: ManifestSelection {
                manifest: PathBuf::from("Gears.toml"),
                app: Some("app".to_owned()),
                env: Some("dev".to_owned()),
            },
            fmt,
            clippy,
            strict,
            dylint,
        }
    }

    fn lint_policy(fmt: bool, clippy: bool, dylint: bool) -> LintPolicy {
        LintPolicy {
            fmt,
            clippy,
            dylint: dylint.then_some(Dylint {
                enabled: true,
                skip: Vec::new(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn defaults_to_manifest_lint_policy() {
        let args = lint_args(false, false, false, false, false);
        let policy = lint_policy(false, true, true);

        let selection = args.selection(&policy);

        assert!(!selection.all);
        assert!(!selection.fmt);
        assert!(selection.clippy);
        assert!(selection.dylint);
    }

    #[test]
    fn explicit_lint_selection_disables_default_all() {
        let args = lint_args(false, false, false, false, true);
        let policy = lint_policy(true, true, false);

        let selection = args.selection(&policy);

        assert!(!selection.all);
        assert!(!selection.fmt);
        assert!(!selection.clippy);
        assert!(selection.dylint);
    }

    #[test]
    fn fmt_selection_is_explicit() {
        let args = lint_args(false, true, false, false, false);
        let policy = lint_policy(false, true, true);

        let selection = args.selection(&policy);

        assert!(!selection.all);
        assert!(selection.fmt);
        assert!(!selection.clippy);
        assert!(!selection.dylint);
    }

    #[test]
    fn strict_with_clippy_is_accepted() {
        let args = lint_args(false, false, true, true, false);

        args.validate(&LintPolicy::default())
            .expect("strict with clippy should be accepted");
    }

    #[test]
    fn strict_with_all_is_accepted() {
        let args = lint_args(true, false, false, true, false);

        args.validate(&LintPolicy::default())
            .expect("strict with all should be accepted");
    }

    #[test]
    fn strict_requires_clippy_or_all() {
        let args = lint_args(false, false, false, true, true);

        let error = args
            .validate(&LintPolicy::default())
            .expect_err("strict should be rejected");

        assert_eq!(
            error.to_string(),
            "`--strict` requires `--clippy` or `--all`"
        );
    }
}
