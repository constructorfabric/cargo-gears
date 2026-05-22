use crate::common::PathConfigArgs;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct DeployArgs {
    #[command(flatten)]
    path_config: PathConfigArgs,
    /// Tag to apply to the generated Docker image
    #[arg(short = 't', long, value_name = "TAG")]
    tag: Option<String>,
    /// Cargo manifest to build instead of generating a server project
    #[arg(short = 'm', long, value_name = "Cargo.toml")]
    manifest: Option<PathBuf>,
    /// By default, builds in release mode. Use this for debug mode.
    #[arg(long)]
    debug: bool,
    /// Dockerfile path to use instead of the default
    #[arg(long)]
    dockerfile: Option<PathBuf>,
    /// Dockerfile ARG override in KEY=VALUE form. Can be repeated.
    #[arg(long = "args", value_name = "KEY=VALUE")]
    args: Vec<cyberware_cli_core::deploy::DockerBuildArg>,
}

impl DeployArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::deploy::DeployArgs {
            path_config: self.path_config.into(),
            tag: self.tag,
            manifest: self.manifest,
            debug: self.debug,
            dockerfile: self.dockerfile,
            args: self.args,
        }
        .run()
    }
}
