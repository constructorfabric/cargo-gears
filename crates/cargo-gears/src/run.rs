use crate::common::{BuildRunArgs, ordered_bool};
use cargo_gears_core::run::RunOutcome;
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
        let watch_override = ordered_bool(self.watch, self.no_watch);

        loop {
            let resolved = self.br_args.resolve()?;
            let watch = watch_override.unwrap_or(resolved.watch_policy.enabled);

            let params = cargo_gears_core::run::RunParams {
                workspace_root: resolved.workspace_root,
                generated_dir: resolved.generated_dir,
                generated_name: resolved.generated_name,
                config_path: resolved.config_path,
                manifest_path: resolved.manifest_path,
                dependencies: resolved.dependencies,
                otel: resolved.otel,
                fips: resolved.fips,
                release: resolved.release,
                clean: resolved.clean,
                dry_run: resolved.dry_run,
                watch,
                watch_policy: resolved.watch_policy,
            };

            match params.run()? {
                RunOutcome::Rerun => {}
                RunOutcome::Stop => break Ok(()),
            }
        }
    }
}
