use crate::common::{ManifestTargetArgs, WorkspacePath};
use clap::Args;

#[derive(Args)]
pub struct LintArgs {
    /// Run all available lint rules
    #[arg(long)]
    all: bool,
    #[command(flatten)]
    pub workspace: WorkspacePath,
    #[command(flatten)]
    pub manifest: ManifestTargetArgs,
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
}

impl LintArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cargo_gears_core::lint::LintParams::from(self).run()
    }
}

impl From<LintArgs> for cargo_gears_core::lint::LintParams {
    fn from(args: LintArgs) -> Self {
        Self {
            all: args.all,
            path: args.workspace.path,
            manifest: args.manifest.into_selection(),
            fmt: args.fmt,
            clippy: args.clippy,
            strict: args.strict,
            dylint: args.dylint,
        }
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
