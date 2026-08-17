use crate::common::{ManifestTargetArgs, WorkspacePath};
use clap::Args;

#[derive(Args)]
pub struct LintArgs {
    /// Run all available lint rules
    #[arg(long)]
    all: bool,
    #[command(flatten)]
    workspace: WorkspacePath,
    #[command(flatten)]
    manifest: ManifestTargetArgs,
    /// Check whether the workspace is formatted with `cargo fmt`.
    #[arg(long)]
    fmt: bool,
    /// Run recommended clippy rules. Follows Cargo.toml exceptions if present.
    #[arg(long)]
    clippy: bool,
    /// Strict mode. Throws an error if any lint rule is triggered.
    #[arg(long)]
    strict: bool,
    /// Run extra lint rules made for gears modules.
    #[arg(long)]
    dylint: bool,
    /// Restrict linting to specific workspace package(s). Repeatable.
    /// When omitted, the whole workspace is linted.
    #[arg(short = 'P', long = "package", value_name = "SPEC")]
    package: Vec<String>,
    /// Also lint every workspace crate that depends on the selected package(s).
    /// Requires at least one `-P/--package`; expands to the reverse-dependency closure.
    #[arg(long = "include-dependents")]
    include_dependents: bool,
    /// Require Cargo.lock is up to date
    #[arg(long, action = clap::ArgAction::SetTrue)]
    locked: bool,
    /// List available lint rules instead of running them.
    /// Combine with `--dylint` to list only dylint rules.
    #[arg(long)]
    list: bool,
}

impl LintArgs {
    const fn has_explicit_selection(&self) -> bool {
        self.all || self.fmt || self.clippy || self.dylint
    }

    /// Resolve manifest + CLI overrides into a fully-resolved `LintParams`.
    pub fn resolve(self) -> anyhow::Result<cargo_gears_core::lint::LintParams> {
        // `--list` short-circuits: print available lints and exit.
        if self.list {
            return Ok(cargo_gears_core::lint::LintParams {
                workspace_root: std::path::PathBuf::new(),
                fmt: false,
                clippy: false,
                strict: false,
                dylint: self.dylint || self.all,
                dylint_skip: Vec::new(),
                packages: Vec::new(),
                include_dependents: false,
                locked: false,
                list: true,
            });
        }

        let workspace_path =
            cargo_gears_core::common::resolve_workspace_path(self.workspace.path.as_deref())?;

        let explicit = self.has_explicit_selection();
        let resolved = self.manifest.into_selection().resolve(&workspace_path)?;
        let policy = &resolved.lint;

        let (fmt, clippy, dylint) = if explicit {
            let all = self.all;
            (self.fmt || all, self.clippy || all, self.dylint || all)
        } else {
            (
                policy.fmt,
                policy.clippy,
                policy.dylint.as_ref().is_some_and(|d| d.enabled),
            )
        };

        if self.strict && !clippy {
            anyhow::bail!("`--strict` requires `--clippy` or `--all`");
        }

        let dylint_skip = resolved
            .lint
            .dylint
            .as_ref()
            .map_or_else(Vec::new, |d| d.skip.clone());

        Ok(cargo_gears_core::lint::LintParams {
            workspace_root: resolved.workspace_root,
            fmt,
            clippy,
            strict: self.strict,
            dylint,
            dylint_skip,
            packages: self.package,
            include_dependents: self.include_dependents,
            locked: self.locked,
            list: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LintArgs;
    use clap::Parser;
    use std::fs;
    use tempfile::TempDir;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        lint: LintArgs,
    }

    fn parse(temp: &TempDir, extra: &[&str]) -> LintArgs {
        let p = temp.path().to_str().expect("temp path should be UTF-8");
        let mut args = vec!["test", "-p", p, "--app", "app", "--env", "dev"];
        args.extend(extra);
        TestCli::try_parse_from(args).expect("should parse").lint
    }

    fn write_workspace(temp: &TempDir, manifest: &str) {
        fs::write(temp.path().join("Gears.toml"), manifest).expect("write manifest");
        fs::create_dir_all(temp.path().join("config")).expect("create config dir");
        fs::write(temp.path().join("config/app-dev.yml"), "server: {}\n").expect("write config");
    }

    const MINIMAL: &str = "[apps.app.dev]\nconfig = \"app-dev.yml\"\ngears = []\n";

    #[test]
    fn parses_default_lint_args() {
        let cli = TestCli::try_parse_from(["gears", "--app", "app1", "--env", "dev"])
            .expect("lint args should parse");

        assert!(!cli.lint.all);
        assert!(!cli.lint.fmt);
        assert!(!cli.lint.clippy);
        assert!(!cli.lint.strict);
        assert!(!cli.lint.dylint);
    }

    #[test]
    fn defaults_to_manifest_lint_policy() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            &format!(
                "{MINIMAL}\
                 [apps.app.dev.lint]\n\
                 fmt = false\n\
                 clippy = true\n\
                 [apps.app.dev.lint.dylint]\n\
                 enabled = true\n"
            ),
        );

        let resolved = parse(&temp, &[]).resolve().expect("resolve");

        assert!(!resolved.fmt);
        assert!(resolved.clippy);
        assert!(resolved.dylint);
    }

    #[test]
    fn explicit_selection_overrides_manifest_policy() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            &format!(
                "{MINIMAL}\
                 [apps.app.dev.lint]\n\
                 fmt = true\n\
                 clippy = true\n"
            ),
        );

        let resolved = parse(&temp, &["--dylint"]).resolve().expect("resolve");

        assert!(!resolved.fmt);
        assert!(!resolved.clippy);
        assert!(resolved.dylint);
    }

    #[test]
    fn all_enables_every_lint() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        let resolved = parse(&temp, &["--all"]).resolve().expect("resolve");

        assert!(resolved.fmt);
        assert!(resolved.clippy);
        assert!(resolved.dylint);
    }

    #[test]
    fn strict_requires_clippy_or_all() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        let err = parse(&temp, &["--strict", "--dylint"])
            .resolve()
            .expect_err("strict without clippy should fail");

        assert_eq!(err.to_string(), "`--strict` requires `--clippy` or `--all`");
    }

    #[test]
    fn strict_with_clippy_is_accepted() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        parse(&temp, &["--strict", "--clippy"])
            .resolve()
            .expect("strict with clippy should succeed");
    }

    #[test]
    fn strict_with_all_is_accepted() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        parse(&temp, &["--strict", "--all"])
            .resolve()
            .expect("strict with all should succeed");
    }

    #[test]
    fn dylint_skip_from_manifest() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            &format!(
                "{MINIMAL}\
                 [apps.app.dev.lint.dylint]\n\
                 enabled = true\n\
                 skip = [\"some_lint\", \"other_lint\"]\n"
            ),
        );

        let resolved = parse(&temp, &[]).resolve().expect("resolve");

        assert_eq!(resolved.dylint_skip, vec!["some_lint", "other_lint"]);
    }

    #[test]
    fn list_flag_skips_manifest_resolution() {
        // --list should resolve without a manifest or workspace path.
        let cli =
            TestCli::try_parse_from(["gears", "--app", "a", "--env", "e", "--list", "--dylint"])
                .expect("should parse");
        let resolved = cli.lint.resolve().expect("resolve");

        assert!(resolved.list);
        assert!(resolved.dylint);
        assert!(!resolved.fmt);
        assert!(!resolved.clippy);
    }

    #[test]
    fn list_without_dylint_sets_dylint_false() {
        let cli = TestCli::try_parse_from(["gears", "--app", "a", "--env", "e", "--list"])
            .expect("should parse");
        let resolved = cli.lint.resolve().expect("resolve");

        assert!(resolved.list);
        assert!(!resolved.dylint);
    }

    #[test]
    fn packages_default_to_empty() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        let resolved = parse(&temp, &["--clippy"]).resolve().expect("resolve");

        assert!(resolved.packages.is_empty());
    }

    #[test]
    fn repeated_package_flags_are_collected() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        let resolved = parse(
            &temp,
            &["--clippy", "--package", "crate-a", "-P", "crate-b"],
        )
        .resolve()
        .expect("resolve");

        assert_eq!(resolved.packages, vec!["crate-a", "crate-b"]);
    }

    #[test]
    fn include_dependents_defaults_to_false() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        let resolved = parse(&temp, &["--clippy", "-P", "crate-a"])
            .resolve()
            .expect("resolve");

        assert!(!resolved.include_dependents);
    }

    #[test]
    fn include_dependents_flag_is_threaded() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        let resolved = parse(
            &temp,
            &["--clippy", "-P", "crate-a", "--include-dependents"],
        )
        .resolve()
        .expect("resolve");

        assert!(resolved.include_dependents);
        assert_eq!(resolved.packages, vec!["crate-a"]);
    }

    #[test]
    fn packages_are_carried_through_dylint_selection() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(&temp, MINIMAL);

        let resolved = parse(&temp, &["--dylint", "-P", "crate-a"])
            .resolve()
            .expect("resolve");

        assert!(resolved.dylint);
        assert_eq!(resolved.packages, vec!["crate-a"]);
    }
}
