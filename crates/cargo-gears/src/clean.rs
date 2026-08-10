use crate::common::{ManifestTargetArgs, WorkspacePath};
use cargo_gears_core::clean::CleanParamsBuilder;
use clap::Args;

#[derive(Args)]
pub struct CleanArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    #[command(flatten)]
    manifest: ManifestTargetArgs,
}

impl CleanArgs {
    pub fn resolve(self) -> anyhow::Result<cargo_gears_core::clean::CleanParams> {
        CleanParamsBuilder::new(self.manifest.manifest_path.manifest)
            .workspace_path(self.workspace.path)
            .app(self.manifest.app)
            .env(self.manifest.env)
            .build()
    }
}
