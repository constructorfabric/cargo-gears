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
            .otel(args.otel)
            .no_otel(args.no_otel)
            .fips(args.fips)
            .no_fips(args.no_fips)
            .release(args.release)
            .no_release(args.no_release)
            .clean(args.clean)
            .no_clean(args.no_clean)
            .dry_run(args.dry_run)
            .build()
    }
}
