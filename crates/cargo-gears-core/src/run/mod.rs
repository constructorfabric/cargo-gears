mod run_loop;
mod watch;

use crate::gears_parser::CargoTomlDependencies;
use crate::manifest::WatchPolicy;
use crate::run::run_loop::RunSignal;
use std::path::PathBuf;

/// Fully-resolved parameters for a single run iteration.
#[derive(Debug)]
pub struct RunParams {
    /// Resolved workspace root path.
    pub workspace_root: PathBuf,
    /// Resolved generated output directory.
    pub generated_dir: PathBuf,
    /// Resolved generated project name.
    pub generated_name: String,
    /// Resolved config file path.
    pub config_path: PathBuf,
    /// Resolved manifest path (needed by watch for change detection).
    pub manifest_path: PathBuf,
    /// Resolved workspace dependencies.
    pub dependencies: CargoTomlDependencies,
    /// Effective otel flag (CLI override already applied).
    pub otel: bool,
    /// Effective FIPS flag.
    pub fips: bool,
    /// Effective release flag.
    pub release: bool,
    /// Whether to remove Cargo.lock before running.
    pub clean: bool,
    /// Print the resolved generation model without running.
    pub dry_run: bool,
    /// Whether to watch for changes.
    pub watch: bool,
    /// Watch policy from manifest.
    pub watch_policy: WatchPolicy,
}

/// Outcome of a single run iteration.
pub enum RunOutcome {
    /// The run loop requests a re-run (e.g. file changed during watch).
    Rerun,
    /// The run completed and should stop.
    Stop,
}

impl RunParams {
    /// Execute a single run iteration. Returns `Rerun` if the watch loop
    /// detected changes and the caller should re-resolve and call again.
    pub fn run(self) -> anyhow::Result<RunOutcome> {
        if self.clean {
            crate::common::remove_from_file_structure(
                &self.generated_dir,
                &self.generated_name,
                "Cargo.lock",
            )?;
        }

        if self.dry_run {
            let generated = crate::common::generate_server_structure(
                &self.workspace_root,
                &self.generated_dir,
                &self.generated_name,
                &self.dependencies,
            )?;
            generated.print_json()?;
            return Ok(RunOutcome::Stop);
        }

        let rl = run_loop::RunLoop::new(
            self.generated_dir,
            self.workspace_root,
            self.config_path,
            self.generated_name,
            self.manifest_path,
            self.watch_policy,
        )
        .with_dependencies(self.dependencies);
        run_loop::OTEL.store(self.otel, std::sync::atomic::Ordering::Relaxed);
        run_loop::FIPS.store(self.fips, std::sync::atomic::Ordering::Relaxed);
        run_loop::RELEASE.store(self.release, std::sync::atomic::Ordering::Relaxed);

        match rl.run(self.watch)? {
            RunSignal::Rerun => Ok(RunOutcome::Rerun),
            RunSignal::Stop => Ok(RunOutcome::Stop),
        }
    }
}
