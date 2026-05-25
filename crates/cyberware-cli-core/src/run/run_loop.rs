use crate::{app_config, common};
use anyhow::{Context, bail};
use notify::{RecursiveMode, Watcher};
use std::collections::HashSet;
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
    workspace_path: PathBuf,
    config_path: PathBuf,
    project_name: String,
}

pub(super) static OTEL: AtomicBool = AtomicBool::new(false);
pub(super) static FIPS: AtomicBool = AtomicBool::new(false);
pub(super) static RELEASE: AtomicBool = AtomicBool::new(false);

impl RunLoop {
    pub(super) const fn new(
        workspace_path: PathBuf,
        config_path: PathBuf,
        project_name: String,
    ) -> Self {
        Self {
            workspace_path,
            config_path,
            project_name,
        }
    }

    pub(super) fn run(&self, watch: bool) -> anyhow::Result<RunSignal> {
        let workspace_path = &self.workspace_path;
        let dependencies =
            common::get_config(workspace_path, &self.config_path)?.create_dependencies()?;
        common::generate_server_structure(workspace_path, &self.project_name, &dependencies)?;

        let cargo_dir = common::generated_project_dir(workspace_path, &self.project_name);

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

        // On Linux and other systems using inotify, when editors perform atomic saves
        // (write to temporary file, then rename), the rename event is reported at the directory level,
        // not the file level. File-level watches can therefore miss these events and fail to detect config changes.
        // Watching the parent directory is the documented best practice.
        let config_parent = self
            .config_path
            .parent()
            .context("config path has no parent directory")?;
        let workspace_manifest = workspace_path.join("Cargo.toml");
        watcher
            .watch(config_parent, RecursiveMode::NonRecursive)
            .context("failed to watch config directory")?;
        if config_parent != workspace_path.as_path() {
            watcher
                .watch(workspace_path, RecursiveMode::NonRecursive)
                .context("failed to watch workspace directory")?;
        }

        // Watch dependency paths that have `path` set
        let mut watched_paths = watch_dependency_paths(&dependencies, &mut watcher, workspace_path);
        let mut current_deps = dependencies;

        // Event loop - runs until the watcher channel closes
        while let Ok(res_event) = fs_rx.recv() {
            let event = match res_event {
                Ok(event) => event,
                Err(err) => {
                    eprintln!("file watcher error: {err}");
                    continue;
                }
            };
            let is_config_change = event.paths.contains(&self.config_path)
                && matches!(
                    event.kind,
                    notify::EventKind::Modify(_)
                        | notify::EventKind::Create(_)
                        | notify::EventKind::Remove(_)
                );
            let is_workspace_manifest_change = event.paths.contains(&workspace_manifest)
                && matches!(
                    event.kind,
                    notify::EventKind::Modify(_)
                        | notify::EventKind::Create(_)
                        | notify::EventKind::Remove(_)
                );

            if is_config_change || is_workspace_manifest_change {
                match common::get_config(workspace_path, &self.config_path)
                    .and_then(app_config::AppConfig::create_dependencies)
                {
                    Ok(new_deps) => {
                        if new_deps != current_deps {
                            if let Err(e) = common::generate_server_structure(
                                workspace_path,
                                &self.project_name,
                                &new_deps,
                            ) {
                                eprintln!("failed to regenerate server structure: {e}");
                            } else {
                                // Reconcile watched dependency paths
                                let new_watched = collect_dep_paths(&new_deps, workspace_path);
                                for old in watched_paths.difference(&new_watched) {
                                    if let Err(err) = watcher.unwatch(old) {
                                        eprintln!("failed to unwatch {}: {err}", old.display());
                                        _ = signal_tx.send(RunSignal::Stop);
                                        runner_handle.join().map_err(|e| {
                                            anyhow::anyhow!("runner thread panicked: {e:?}")
                                        })?;
                                        return Ok(RunSignal::Rerun);
                                    }
                                }
                                for new_p in new_watched.difference(&watched_paths) {
                                    if let Err(err) = watcher.watch(new_p, RecursiveMode::Recursive)
                                    {
                                        eprintln!("failed to watch {}: {err}", new_p.display());
                                        _ = signal_tx.send(RunSignal::Stop);
                                        runner_handle.join().map_err(|e| {
                                            anyhow::anyhow!("runner thread panicked: {e:?}")
                                        })?;
                                        return Ok(RunSignal::Rerun);
                                    }
                                }
                                watched_paths = new_watched;
                                current_deps = new_deps;
                            }
                        }
                        _ = signal_tx.send(RunSignal::Rerun);
                    }
                    Err(e) => eprintln!("failed to reload config: {e}"),
                }
            } else {
                // A watched dependency path changed
                _ = signal_tx.send(RunSignal::Rerun);
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

fn collect_dep_paths(
    deps: &crate::module_parser::CargoTomlDependencies,
    base_path: &Path,
) -> HashSet<PathBuf> {
    deps.values()
        .filter_map(|d| d.path.as_ref())
        .map(|p| base_path.join(p))
        .collect()
}

fn watch_dependency_paths(
    deps: &crate::module_parser::CargoTomlDependencies,
    watcher: &mut impl Watcher,
    base_path: &Path,
) -> HashSet<PathBuf> {
    let paths = collect_dep_paths(deps, base_path);
    for p in &paths {
        if let Err(e) = watcher.watch(p, RecursiveMode::Recursive) {
            eprintln!("failed to watch {}: {e}", p.display());
        }
    }
    paths
}
