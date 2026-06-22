use crate::common::BuildRunArgs;
use cargo_gears_core::run::{RunOutcome, RunParamsBuilder};
use clap::{ArgAction, Args};

#[derive(Args)]
pub struct RunArgs {
    /// Watch for changes
    #[arg(short = 'w', long, action = ArgAction::SetTrue, conflicts_with = "no_watch")]
    watch: Option<bool>,
    /// Do not watch for changes
    #[arg(long = "no-watch", action = ArgAction::SetTrue, conflicts_with = "watch")]
    no_watch: Option<bool>,
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
                .watch(self.watch)
                .no_watch(self.no_watch)
                .build()?;

            match params.run()? {
                RunOutcome::Rerun => {}
                RunOutcome::Stop => break Ok(()),
            }
        }
    }
}
