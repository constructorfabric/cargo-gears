use crate::common::BuildRunArgs;
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[command(flatten)]
    pub build_run_args: BuildRunArgs,
}

impl BuildArgs {
    /// Resolve manifest + CLI overrides into a fully-resolved `BuildParams`.
    pub fn resolve(self) -> anyhow::Result<cargo_gears_core::build::BuildParams> {
        let resolved = self.build_run_args.resolve()?;
        Ok(cargo_gears_core::build::BuildParams {
            workspace_root: resolved.workspace_root,
            generated_dir: resolved.generated_dir,
            generated_name: resolved.generated_name,
            config_path: resolved.config_path,
            dependencies: resolved.dependencies,
            otel: resolved.otel,
            fips: resolved.fips,
            release: resolved.release,
            clean: resolved.clean,
            dry_run: resolved.dry_run,
        })
    }
}
