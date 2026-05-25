use crate::common::{self, BuildRunArgs};
use anyhow::{Context, bail};

#[derive(Debug, Eq, PartialEq)]
pub struct BuildArgs {
    pub build_run_args: BuildRunArgs,
}

impl BuildArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.build_run_args
            .path_config
            .with_workspace_dir(|workspace_path, config_path| {
                let project_name = common::resolve_generated_project_name(
                    config_path,
                    self.build_run_args.name.as_deref(),
                )?;
                if self.build_run_args.clean {
                    common::remove_from_file_structure(
                        workspace_path,
                        &project_name,
                        "Cargo.lock",
                    )?;
                }

                let dependencies =
                    common::get_config(workspace_path, config_path)?.create_dependencies()?;
                common::generate_server_structure(workspace_path, &project_name, &dependencies)?;

                let cargo_dir = common::generated_project_dir(workspace_path, &project_name);
                let status = common::cargo_command(
                    "build",
                    &cargo_dir,
                    config_path,
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
            })
    }
}
