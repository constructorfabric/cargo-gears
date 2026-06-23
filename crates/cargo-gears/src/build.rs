use crate::common::BuildRunArgs;
use cargo_gears_core::build::BuildParamsBuilder;
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[command(flatten)]
    build_run_args: BuildRunArgs,
}

impl BuildArgs {
    /// Resolve manifest + CLI overrides into a fully-resolved `BuildParams`.
    pub fn resolve(self) -> anyhow::Result<cargo_gears_core::build::BuildParams> {
        let args = self.build_run_args;
        BuildParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .build()
    }
}
