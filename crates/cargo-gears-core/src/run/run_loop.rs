use super::watch::{WatchAction, WatchPlan, WatchPlanInputs};
use crate::common;
use crate::manifest::WatchPolicy;
use anyhow::{Context, bail};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::Duration;

pub(super) enum RunSignal {
    Rerun,
    Stop,
}

pub(super) struct RunLoop {
    generated_dir: PathBuf,
    workspace_path: PathBuf,
    config_path: PathBuf,
    project_name: String,
    manifest_path: PathBuf,
    watch_policy: WatchPolicy,
    dependencies: Option<crate::gears_parser::CargoTomlDependencies>,
}

pub(super) static OTEL: AtomicBool = AtomicBool::new(false);
pub(super) static FIPS: AtomicBool = AtomicBool::new(false);
pub(super) static RELEASE: AtomicBool = AtomicBool::new(false);

impl RunLoop {
    pub(super) const fn new(
        generated_dir: PathBuf,
        workspace_path: PathBuf,
        config_path: PathBuf,
        project_name: String,
        manifest_path: PathBuf,
        watch_policy: WatchPolicy,
    ) -> Self {
        Self {
            generated_dir,
            workspace_path,
            config_path,
            project_name,
            manifest_path,
            watch_policy,
            dependencies: None,
        }
    }

    pub(super) fn with_dependencies(
        mut self,
        dependencies: crate::gears_parser::CargoTomlDependencies,
    ) -> Self {
        self.dependencies = Some(dependencies);
        self
    }

    pub(super) fn run(&self, watch: bool) -> anyhow::Result<RunSignal> {
        let workspace_path = &self.workspace_path;
        let dependencies = self.dependencies.as_ref().map_or_else(
            || common::get_config(workspace_path, &self.config_path)?.create_dependencies(),
            |dependencies| Ok(dependencies.clone()),
        )?;
        common::generate_server_structure(
            workspace_path,
            &self.generated_dir,
            &self.project_name,
            &dependencies,
        )?;

        let cargo_dir = common::generated_project_dir(&self.generated_dir, &self.project_name);

        if !watch {
            let status = cargo_run(&cargo_dir, &self.config_path)?
                .status()
                .context("failed to run cargo")?;
            if !status.success() {
                bail!("cargo run exited with {status}");
            }
            return Ok(RunSignal::Stop);
        }

        // -- watch mode --

        let (signal_tx, signal_rx) = mpsc::channel::<RunSignal>();

        // Spawn cargo-run loop in a dedicated thread
        let cargo_dir_clone = cargo_dir;
        let config_path = self.config_path.clone();
        let runner_handle = std::thread::spawn(move || {
            cargo_run_loop(&cargo_dir_clone, &config_path, &signal_rx);
        });

        // File-system watcher
        let (fs_tx, fs_rx) = mpsc::channel();
        let mut watcher =
            notify::recommended_watcher(fs_tx).context("failed to create file watcher")?;

        let watch_plan = WatchPlan::from_policy(
            &self.watch_policy,
            WatchPlanInputs {
                workspace_path,
                manifest_path: &self.manifest_path,
                config_path: &self.config_path,
                dependencies: &dependencies,
            },
        )?;
        watch_plan.watch(&mut watcher)?;

        // Event loop - runs until the watcher channel closes
        while let Ok(res_event) = fs_rx.recv() {
            let event = match res_event {
                Ok(event) => event,
                Err(err) => {
                    eprintln!("file watcher error: {err}");
                    continue;
                }
            };

            match watch_plan.action_for_event(&event)? {
                Some(WatchAction::Regenerate) => {
                    _ = signal_tx.send(RunSignal::Stop);
                    runner_handle
                        .join()
                        .map_err(|e| anyhow::anyhow!("runner thread panicked: {e:?}"))?;
                    return Ok(RunSignal::Rerun);
                }
                Some(WatchAction::Restart) => {
                    _ = signal_tx.send(RunSignal::Rerun);
                }
                None => {}
            }
        }

        // Watcher channel closed - shut down the runner
        _ = signal_tx.send(RunSignal::Stop);
        runner_handle
            .join()
            .map_err(|e| anyhow::anyhow!("runner thread panicked: {e:?}"))?;

        Ok(RunSignal::Stop)
    }
}

fn cargo_run(path: &Path, config_path: &Path) -> anyhow::Result<Command> {
    let otel = OTEL.load(std::sync::atomic::Ordering::Relaxed);
    let fips = FIPS.load(std::sync::atomic::Ordering::Relaxed);
    let release = RELEASE.load(std::sync::atomic::Ordering::Relaxed);
    common::cargo_command("run", path, config_path, otel, fips, release)
}

fn cargo_run_loop(cargo_dir: &Path, config_path: &Path, signal_rx: &mpsc::Receiver<RunSignal>) {
    'outer: loop {
        let mut child = match cargo_run(cargo_dir, config_path)
            .and_then(|mut cmd| cmd.spawn().context("failed to spawn cargo run"))
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("failed to spawn cargo run: {e}");
                match signal_rx.recv() {
                    Ok(RunSignal::Rerun) => continue 'outer,
                    _ => return,
                }
            }
        };

        let rerun = loop {
            match child.try_wait() {
                Ok(Some(_)) => break false,
                Ok(None) => {}
                Err(e) => {
                    eprintln!("error checking child status: {e}");
                    break false;
                }
            }

            match signal_rx.try_recv() {
                Ok(RunSignal::Rerun) => {
                    // Drain extra reruns; honor a queued Stop.
                    let mut stop = false;
                    loop {
                        match signal_rx.try_recv() {
                            Ok(RunSignal::Rerun) => {}
                            Ok(RunSignal::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                                stop = true;
                                break;
                            }
                            Err(mpsc::TryRecvError::Empty) => break,
                        }
                    }
                    let _ = child.kill();
                    let _ = child.wait();
                    if stop {
                        return;
                    }
                    break true;
                }
                Ok(RunSignal::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }

            std::thread::sleep(Duration::from_millis(100));
        };

        if rerun {
            continue 'outer;
        }

        // Child exited on its own, wait for a signal before restarting
        match signal_rx.recv() {
            Ok(RunSignal::Rerun) => {}
            _ => return,
        }
    }
}
