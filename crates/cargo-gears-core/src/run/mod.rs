mod run_loop;
mod watch;

use crate::manifest::{BuildProfile, ManifestSelection, WatchPolicy};
use crate::run::run_loop::RunSignal;
use std::path::PathBuf;

/// Fully-resolved parameters for a single run iteration.
#[derive(Debug)]
pub struct RunParams {
    /// Shared build/run parameters.
    pub build_run_args: crate::common::BuildRunParams,
    /// Resolved manifest path (needed by watch for change detection).
    pub manifest_path: PathBuf,
    /// Whether to watch for changes.
    pub watch: bool,
    /// Watch policy from manifest.
    pub watch_policy: WatchPolicy,
}

/// Builder for constructing `RunParams` with manifest resolution and CLI overrides.
pub struct RunParamsBuilder {
    workspace_path: Option<PathBuf>,
    manifest: PathBuf,
    app: Option<String>,
    env: Option<String>,
    name: Option<String>,
    otel: Option<bool>,
    no_otel: Option<bool>,
    fips: Option<bool>,
    no_fips: Option<bool>,
    release: Option<bool>,
    no_release: Option<bool>,
    clean: Option<bool>,
    no_clean: Option<bool>,
    dry_run: bool,
    watch: Option<bool>,
    no_watch: Option<bool>,
}

impl RunParamsBuilder {
    #[must_use]
    pub const fn new(manifest: PathBuf) -> Self {
        Self {
            workspace_path: None,
            manifest,
            app: None,
            env: None,
            name: None,
            otel: None,
            no_otel: None,
            fips: None,
            no_fips: None,
            release: None,
            no_release: None,
            clean: None,
            no_clean: None,
            dry_run: false,
            watch: None,
            no_watch: None,
        }
    }

    #[must_use]
    pub fn workspace_path(mut self, path: Option<PathBuf>) -> Self {
        self.workspace_path = path;
        self
    }

    #[must_use]
    pub fn app(mut self, app: Option<String>) -> Self {
        self.app = app;
        self
    }

    #[must_use]
    pub fn env(mut self, env: Option<String>) -> Self {
        self.env = env;
        self
    }

    #[must_use]
    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    #[must_use]
    pub const fn otel(mut self, otel: Option<bool>) -> Self {
        self.otel = otel;
        self
    }

    #[must_use]
    pub const fn no_otel(mut self, no_otel: Option<bool>) -> Self {
        self.no_otel = no_otel;
        self
    }

    #[must_use]
    pub const fn fips(mut self, fips: Option<bool>) -> Self {
        self.fips = fips;
        self
    }

    #[must_use]
    pub const fn no_fips(mut self, no_fips: Option<bool>) -> Self {
        self.no_fips = no_fips;
        self
    }

    #[must_use]
    pub const fn release(mut self, release: Option<bool>) -> Self {
        self.release = release;
        self
    }

    #[must_use]
    pub const fn no_release(mut self, no_release: Option<bool>) -> Self {
        self.no_release = no_release;
        self
    }

    #[must_use]
    pub const fn clean(mut self, clean: Option<bool>) -> Self {
        self.clean = clean;
        self
    }

    #[must_use]
    pub const fn no_clean(mut self, no_clean: Option<bool>) -> Self {
        self.no_clean = no_clean;
        self
    }

    #[must_use]
    pub const fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    #[must_use]
    pub const fn watch(mut self, watch: Option<bool>) -> Self {
        self.watch = watch;
        self
    }

    #[must_use]
    pub const fn no_watch(mut self, no_watch: Option<bool>) -> Self {
        self.no_watch = no_watch;
        self
    }

    pub fn build(self) -> anyhow::Result<RunParams> {
        let workspace_root = crate::common::resolve_workspace_path(self.workspace_path.as_deref())?;
        let manifest_selection = ManifestSelection {
            manifest: self.manifest,
            app: self.app,
            env: self.env,
        };
        let resolved = manifest_selection.resolve(&workspace_root)?;

        let otel =
            crate::common::ordered_bool(self.otel, self.no_otel).unwrap_or(resolved.run.otel);
        let fips =
            crate::common::ordered_bool(self.fips, self.no_fips).unwrap_or(resolved.run.fips);
        let release = crate::common::ordered_bool(self.release, self.no_release).unwrap_or(
            matches!(resolved.build.profile, Some(BuildProfile::Release)),
        );
        let clean = crate::common::ordered_bool(self.clean, self.no_clean)
            .unwrap_or_else(|| resolved.build.clean.unwrap_or(release));
        let watch = crate::common::ordered_bool(self.watch, self.no_watch)
            .unwrap_or(resolved.run.watch.enabled);

        Ok(RunParams {
            build_run_args: crate::common::BuildRunParams {
                workspace_root: resolved.workspace_root,
                generated_dir: resolved.generated_dir,
                generated_name: self.name.unwrap_or(resolved.generated_name),
                config_path: resolved.config_path,
                dependencies: resolved.dependencies,
                otel,
                fips,
                release,
                clean,
                dry_run: self.dry_run,
            },
            manifest_path: resolved.manifest_path,
            watch,
            watch_policy: resolved.run.watch,
        })
    }
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
        if self.build_run_args.clean {
            crate::common::remove_from_file_structure(
                &self.build_run_args.generated_dir,
                &self.build_run_args.generated_name,
                "Cargo.lock",
            )?;
        }

        if self.build_run_args.dry_run {
            let generated = crate::common::generate_server_structure(
                &self.build_run_args.workspace_root,
                &self.build_run_args.generated_dir,
                &self.build_run_args.generated_name,
                &self.build_run_args.dependencies,
            )?;
            generated.print_json()?;
            return Ok(RunOutcome::Stop);
        }

        let rl = run_loop::RunLoop::new(
            self.build_run_args.generated_dir,
            self.build_run_args.workspace_root,
            self.build_run_args.config_path,
            self.build_run_args.generated_name,
            self.manifest_path,
            self.watch_policy,
        )
        .with_dependencies(self.build_run_args.dependencies);
        run_loop::OTEL.store(
            self.build_run_args.otel,
            std::sync::atomic::Ordering::Relaxed,
        );
        run_loop::FIPS.store(
            self.build_run_args.fips,
            std::sync::atomic::Ordering::Relaxed,
        );
        run_loop::RELEASE.store(
            self.build_run_args.release,
            std::sync::atomic::Ordering::Relaxed,
        );

        match rl.run(self.watch)? {
            RunSignal::Rerun => Ok(RunOutcome::Rerun),
            RunSignal::Stop => Ok(RunOutcome::Stop),
        }
    }
}
