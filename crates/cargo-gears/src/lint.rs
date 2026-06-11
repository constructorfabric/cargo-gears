use crate::common::{ManifestTargetArgs, WorkspacePath};
use clap::Args;

#[derive(Args)]
pub struct LintArgs {
    /// Run all available lint rules
    #[arg(long)]
    pub all: bool,
    #[command(flatten)]
    pub workspace: WorkspacePath,
    #[command(flatten)]
    pub manifest: ManifestTargetArgs,
    /// Check whether the workspace is formatted with `cargo fmt`.
    #[arg(long)]
    pub fmt: bool,
    /// Run recommended clippy rules. Follows Cargo.toml exceptions if present.
    #[arg(long)]
    pub clippy: bool,
    /// Strict mode. Throws an error if any lint rule is triggered.
    #[arg(long)]
    pub strict: bool,
    /// Run extra lint rules made for gears modules.
    #[arg(long)]
    pub dylint: bool,
}

impl LintArgs {
    const fn has_explicit_selection(&self) -> bool {
        self.all || self.fmt || self.clippy || self.dylint
    }

    /// Resolve manifest + CLI overrides into a fully-resolved `LintParams`.
    pub fn resolve(self) -> anyhow::Result<cargo_gears_core::lint::LintParams> {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LintArgs;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        lint: LintArgs,
    }

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
}
