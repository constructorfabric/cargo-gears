use crate::common::{BuildRunArgs, ordered_bool};
use cargo_gears_core::run::RunOutcome;
use clap::{ArgAction, Args};

#[derive(Args)]
pub struct RunArgs {
    /// Watch for changes
    #[arg(short = 'w', long, action = ArgAction::SetTrue, conflicts_with = "no_watch")]
    pub watch: bool,
    /// Do not watch for changes
    #[arg(long = "no-watch", action = ArgAction::SetTrue, conflicts_with = "watch")]
    pub no_watch: bool,
    #[command(flatten)]
    pub br_args: BuildRunArgs,
}

impl RunArgs {
    /// Resolve manifest, apply CLI overrides, and run — re-resolving on each
    /// watch loop iteration so manifest changes are picked up.
    pub fn resolve_and_run(self) -> anyhow::Result<()> {
        let watch_override = ordered_bool(self.watch, self.no_watch);
        let workspace_path = cargo_gears_core::common::resolve_workspace_path(
            self.br_args.workspace.path.as_deref(),
        )?;
        let manifest_selection = self.br_args.manifest.into_selection();
        let otel_flag = ordered_bool(self.br_args.otel, self.br_args.no_otel);
        let fips_flag = ordered_bool(self.br_args.fips, self.br_args.no_fips);
        let release_flag = ordered_bool(self.br_args.release, self.br_args.no_release);
        let clean_flag = ordered_bool(self.br_args.clean, self.br_args.no_clean);
        let dry_run = self.br_args.dry_run;

        loop {
            let resolved = manifest_selection.resolve(&workspace_path)?;

            let otel = otel_flag.unwrap_or(resolved.run.otel);
            let fips = fips_flag.unwrap_or(resolved.run.fips);
            let release = release_flag.unwrap_or(matches!(
                resolved.build.profile,
                Some(cargo_gears_core::manifest::BuildProfile::Release)
            ));
            let clean = clean_flag.unwrap_or_else(|| resolved.build.clean.unwrap_or(release));
            let watch = watch_override.unwrap_or(resolved.run.watch.enabled);

            let params = cargo_gears_core::run::RunParams {
                workspace_root: resolved.workspace_root,
                generated_dir: resolved.generated_dir,
                generated_name: resolved.generated_name,
                config_path: resolved.config_path,
                manifest_path: resolved.manifest_path,
                dependencies: resolved.dependencies,
                otel,
                fips,
                release,
                clean,
                dry_run,
                watch,
                watch_policy: resolved.run.watch,
            };

            match params.run()? {
                RunOutcome::Rerun => {}
                RunOutcome::Stop => break Ok(()),
            }
        }
    }
}
