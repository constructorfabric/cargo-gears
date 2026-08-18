use crate::common::{ManifestTargetArgs, WorkspacePath};
use anyhow::Context;
use cargo_gears_core::common;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct DeployArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    #[command(flatten)]
    manifest_target: ManifestTargetArgs,
    /// Tag to apply to the generated Docker image
    #[arg(short = 't', long, value_name = "TAG")]
    tag: Option<String>,
    /// Cargo manifest to build instead of the auto-resolved generated server
    #[arg(short = 'm', long = "manifest", value_name = "Cargo.toml")]
    cargo_manifest: Option<PathBuf>,
    /// Config file override (resolved from Gears.toml when omitted)
    #[arg(short = 'c', long)]
    config: Option<PathBuf>,
    /// By default, builds in release mode. Use this for debug mode.
    #[arg(long)]
    debug: bool,
    /// Dockerfile path to use instead of the default
    #[arg(long)]
    dockerfile: Option<PathBuf>,
    /// Dockerfile ARG override in KEY=VALUE form. Can be repeated.
    #[arg(long = "args", value_name = "KEY=VALUE")]
    args: Vec<cargo_gears_core::deploy::DockerBuildArg>,
}

impl DeployArgs {
    pub fn resolve(self) -> anyhow::Result<cargo_gears_core::deploy::DeployParams> {
        let workspace_root = common::resolve_workspace_path(self.workspace.path.as_deref())?;
        let resolved = self
            .manifest_target
            .into_selection()
            .resolve(&workspace_root)?;

        // Use explicit --manifest or fall back to the generated project's Cargo.toml.
        let cargo_manifest = self.cargo_manifest.unwrap_or_else(|| {
            let project_dir =
                common::generated_project_dir(&resolved.generated_dir, &resolved.generated_name);
            project_dir.join("Cargo.toml")
        });

        // Use explicit --config or fall back to the manifest-resolved config path.
        let config_path = self.config.map_or_else(
            || Ok(resolved.config_path),
            |c| {
                if c.is_absolute() {
                    Ok(c)
                } else {
                    workspace_root
                        .join(&c)
                        .canonicalize()
                        .with_context(|| format!("can't resolve config path {}", c.display()))
                }
            },
        )?;

        Ok(cargo_gears_core::deploy::DeployParams {
            workspace_root: resolved.workspace_root,
            config_path,
            cargo_manifest,
            tag: self.tag,
            debug: self.debug,
            dockerfile: self.dockerfile,
            args: self.args,
        })
    }
}
