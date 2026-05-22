use crate::common::{self, BuildRunArgs};
use anyhow::{Context, bail};

pub struct BuildArgs {
    pub build_run_args: BuildRunArgs,
}

impl BuildArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let (config_path, project_name) = self.build_run_args.resolve_config_and_name()?;

        let dependencies = common::get_config(&config_path)?.create_dependencies()?;
        common::generate_server_structure(&project_name, &dependencies)?;

        let cargo_dir = common::generated_project_dir(&project_name)?;
        let status = common::cargo_command(
            "build",
            &cargo_dir,
            &config_path,
            self.build_run_args.otel,
            self.build_run_args.fips,
            self.build_run_args.release,
        )?
        .status()
        .context("failed to run cargo build")?;

        if !status.success() {
            bail!("cargo build exited with {status}");
        }

        Ok(())
    }
}
