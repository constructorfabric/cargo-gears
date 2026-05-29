mod run_loop;

use crate::common::BuildRunParams;
use crate::run::run_loop::RunSignal;

#[derive(Debug, Eq, PartialEq)]
pub struct RunParams {
    /// Watch for changes
    pub watch: Option<bool>,
    pub br_args: BuildRunParams,
}

impl RunParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_path = crate::common::resolve_workspace_path(self.br_args.path.as_deref())?;
        let resolved = self.br_args.manifest.resolve(&workspace_path)?;

        self.br_args.clean_build(&resolved)?;

        let otel = self.br_args.otel_enabled(&resolved);
        let fips = self.br_args.fips_enabled(&resolved);
        let release = self.br_args.release_build(&resolved);
        let watch = self.watch.unwrap_or(resolved.run.watch.enabled);

        if self.br_args.dry_run {
            let generated = crate::common::generate_server_structure(
                &resolved.workspace_root,
                &resolved.generated_dir,
                &resolved.generated_name,
                &resolved.dependencies,
            )?;
            return generated.print_json();
        }

        let rl = run_loop::RunLoop::new(
            resolved.generated_dir,
            resolved.workspace_root,
            resolved.config_path,
            resolved.generated_name,
        )
        .with_dependencies(resolved.dependencies);
        run_loop::OTEL.store(otel, std::sync::atomic::Ordering::Relaxed);
        run_loop::FIPS.store(fips, std::sync::atomic::Ordering::Relaxed);
        run_loop::RELEASE.store(release, std::sync::atomic::Ordering::Relaxed);

        loop {
            match rl.run(watch)? {
                RunSignal::Rerun => {}
                RunSignal::Stop => break Ok(()),
            }
        }
    }
}
