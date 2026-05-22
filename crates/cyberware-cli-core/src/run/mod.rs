mod run_loop;

use crate::common::BuildRunArgs;
use crate::run::run_loop::RunSignal;

pub struct RunArgs {
    /// Watch for changes
    pub watch: bool,
    pub br_args: BuildRunArgs,
}

impl RunArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let (config_path, project_name) = self.br_args.resolve_config_and_name()?;

        let rl = run_loop::RunLoop::new(config_path, project_name);
        run_loop::OTEL.store(self.br_args.otel, std::sync::atomic::Ordering::Relaxed);
        run_loop::FIPS.store(self.br_args.fips, std::sync::atomic::Ordering::Relaxed);
        run_loop::RELEASE.store(self.br_args.release, std::sync::atomic::Ordering::Relaxed);

        loop {
            match rl.run(self.watch)? {
                RunSignal::Rerun => {}
                RunSignal::Stop => break Ok(()),
            }
        }
    }
}
