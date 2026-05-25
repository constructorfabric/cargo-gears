mod run_loop;

use crate::common::BuildRunArgs;
use crate::run::run_loop::RunSignal;

#[derive(Debug, Eq, PartialEq)]
pub struct RunArgs {
    /// Watch for changes
    pub watch: bool,
    pub br_args: BuildRunArgs,
}

impl RunArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.br_args
            .path_config
            .with_workspace_dir(|workspace_path, config_path| {
                let project_name = crate::common::resolve_generated_project_name(
                    config_path,
                    self.br_args.name.as_deref(),
                )?;
                if self.br_args.clean {
                    crate::common::remove_from_file_structure(
                        workspace_path,
                        &project_name,
                        "Cargo.lock",
                    )?;
                }

                let rl = run_loop::RunLoop::new(
                    workspace_path.to_path_buf(),
                    config_path.to_path_buf(),
                    project_name,
                );
                run_loop::OTEL.store(self.br_args.otel, std::sync::atomic::Ordering::Relaxed);
                run_loop::FIPS.store(self.br_args.fips, std::sync::atomic::Ordering::Relaxed);
                run_loop::RELEASE.store(self.br_args.release, std::sync::atomic::Ordering::Relaxed);

                loop {
                    match rl.run(self.watch)? {
                        RunSignal::Rerun => {}
                        RunSignal::Stop => break Ok(()),
                    }
                }
            })
    }
}
