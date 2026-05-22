use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct LintArgs {
    /// Run all available lint rules
    #[arg(long)]
    all: bool,
    /// Path to the module workspace root
    #[arg(short = 'p', long, value_parser = cyberware_cli_core::common::parse_and_chdir)]
    pub path: Option<PathBuf>,
    /// Check whether the workspace is formatted with `cargo fmt`.
    #[arg(long)]
    fmt: bool,
    /// Run recommended clippy rules. Follows Cargo.toml exceptions if present.
    #[arg(long)]
    clippy: bool,
    /// Strict mode. Throws an error if any lint rule is triggered.
    #[arg(long)]
    strict: bool,
    /// Run extra lint rules made for cyberfabric modules.
    #[arg(long)]
    dylint: bool,
}

impl LintArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::lint::LintArgs {
            all: self.all,
            path: self.path,
            fmt: self.fmt,
            clippy: self.clippy,
            strict: self.strict,
            dylint: self.dylint,
        }
        .run()
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
        let cli = TestCli::try_parse_from(["cyberfabric"]).expect("lint args should parse");

        assert!(!cli.lint.all);
        assert!(!cli.lint.fmt);
        assert!(!cli.lint.clippy);
        assert!(!cli.lint.strict);
        assert!(!cli.lint.dylint);
    }
}
