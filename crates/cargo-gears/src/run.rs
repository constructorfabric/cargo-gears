use crate::common::BuildRunArgs;
use cargo_gears_core::run::{RunOutcome, RunParamsBuilder};
use clap::{ArgAction, Args};

#[derive(Args)]
pub struct RunArgs {
    /// Watch for changes
    #[arg(short = 'w', long, action = ArgAction::SetTrue, conflicts_with = "no_watch")]
    watch: bool,
    /// Do not watch for changes
    #[arg(long = "no-watch", action = ArgAction::SetTrue, conflicts_with = "watch")]
    no_watch: bool,
    #[command(flatten)]
    br_args: BuildRunArgs,
}

impl RunArgs {
    /// Resolve manifest, apply CLI overrides, and run — re-resolving on each
    /// watch loop iteration so manifest changes are picked up.
    pub fn resolve_and_run(self) -> anyhow::Result<()> {
        loop {
            let args = &self.br_args;
            let params = RunParamsBuilder::new(args.manifest.manifest_path.manifest.clone())
                .workspace_path(args.workspace.path.clone())
                .app(args.manifest.app.clone())
                .env(args.manifest.env.clone())
                .otel(args.otel)
                .no_otel(args.no_otel)
                .fips(args.fips)
                .no_fips(args.no_fips)
                .release(args.release)
                .no_release(args.no_release)
                .clean(args.clean)
                .no_clean(args.no_clean)
                .dry_run(args.dry_run)
                .watch(self.watch.then_some(true))
                .no_watch(self.no_watch.then_some(true))
                .build()?;

            match params.run()? {
                RunOutcome::Rerun => {}
                RunOutcome::Stop => break Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RunArgs;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        run: RunArgs,
    }

    fn parse(extra: &[&str]) -> RunArgs {
        let mut args = vec!["test"];
        args.extend(extra);
        TestCli::try_parse_from(args).expect("should parse").run
    }

    #[test]
    fn watch_flag_sets_true() {
        let args = parse(&["-w"]);
        assert!(args.watch);
        assert!(!args.no_watch);
    }

    #[test]
    fn no_watch_flag_sets_true() {
        let args = parse(&["--no-watch"]);
        assert!(!args.watch);
        assert!(args.no_watch);
    }

    #[test]
    fn neither_watch_flag_defaults_to_false() {
        let args = parse(&[]);
        assert!(!args.watch);
        assert!(!args.no_watch);
    }

    #[test]
    fn watch_and_no_watch_conflict() {
        let result =
            TestCli::try_parse_from(["test", "--watch", "--no-watch"]);
        assert!(result.is_err());
    }

    #[test]
    fn watch_converts_to_option_for_builder() {
        let args = parse(&["-w"]);
        assert_eq!(args.watch.then_some(true), Some(true));
        assert_eq!(args.no_watch.then_some(true), None);
    }

    #[test]
    fn no_watch_converts_to_option_for_builder() {
        let args = parse(&["--no-watch"]);
        assert_eq!(args.watch.then_some(true), None);
        assert_eq!(args.no_watch.then_some(true), Some(true));
    }

    #[test]
    fn neither_flag_converts_to_none_for_builder() {
        let args = parse(&[]);
        assert_eq!(args.watch.then_some(true), None);
        assert_eq!(args.no_watch.then_some(true), None);
    }
}
