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

/// Events received by the main watch-mode event loop.
enum WatchEvent {
    /// A file-system event from the watcher.
    Fs(Result<notify::Event, notify::Error>),
    /// The child process exited on its own (not killed by a signal).
    ChildExited,
}

pub(super) struct RunLoop {
    generated_dir: PathBuf,
    workspace_path: PathBuf,
    config_path: PathBuf,
    project_name: String,
    manifest_path: PathBuf,
    watch_policy: WatchPolicy,
    dependencies: crate::gears_parser::CargoTomlDependencies,
}

pub(super) static OTEL: AtomicBool = AtomicBool::new(false);
pub(super) static FIPS: AtomicBool = AtomicBool::new(false);
pub(super) static RELEASE: AtomicBool = AtomicBool::new(false);
pub(super) static LOCKED: AtomicBool = AtomicBool::new(false);

impl RunLoop {
    pub(super) const fn new(
        generated_dir: PathBuf,
        workspace_path: PathBuf,
        config_path: PathBuf,
        project_name: String,
        manifest_path: PathBuf,
        watch_policy: WatchPolicy,
        dependencies: crate::gears_parser::CargoTomlDependencies,
    ) -> Self {
        Self {
            generated_dir,
            workspace_path,
            config_path,
            project_name,
            manifest_path,
            watch_policy,
            dependencies,
        }
    }

    pub(super) fn run(&self, watch: bool) -> anyhow::Result<RunSignal> {
        let workspace_path = &self.workspace_path;
        let dependencies = &self.dependencies;
        common::generate_server_structure(
            workspace_path,
            &self.generated_dir,
            &self.project_name,
            dependencies,
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
        let (event_tx, event_rx) = mpsc::channel::<WatchEvent>();

        // Spawn cargo-run loop in a dedicated thread
        let cargo_dir_clone = cargo_dir;
        let config_path = self.config_path.clone();
        let runner_tx = event_tx.clone();
        let runner_handle = std::thread::spawn(move || {
            cargo_run_loop(&cargo_dir_clone, &config_path, &signal_rx, &runner_tx);
        });

        // File-system watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = event_tx.send(WatchEvent::Fs(res));
        })
        .context("failed to create file watcher")?;

        let watch_plan = WatchPlan::from_policy(
            &self.watch_policy,
            WatchPlanInputs {
                workspace_path,
                manifest_path: &self.manifest_path,
                config_path: &self.config_path,
                dependencies,
            },
        )?;
        watch_plan.watch(&mut watcher)?;

        // Unified event loop: handles both file-system events and child-exit
        // notifications through a single channel.
        while let Ok(watch_event) = event_rx.recv() {
            match watch_event {
                WatchEvent::ChildExited => {
                    runner_handle
                        .join()
                        .map_err(|e| anyhow::anyhow!("runner thread panicked: {e:?}"))?;
                    return Ok(RunSignal::Stop);
                }
                WatchEvent::Fs(res_event) => {
                    let event = match res_event {
                        Ok(event) => event,
                        Err(err) => {
                            eprintln!("file watcher error: {err}");
                            continue;
                        }
                    };

                    match watch_plan.action_for_event(&event) {
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
            }
        }

        // Event channel closed - shut down the runner
        _ = signal_tx.send(RunSignal::Stop);
        runner_handle
            .join()
            .map_err(|e| anyhow::anyhow!("runner thread panicked: {e:?}"))?;

        Ok(RunSignal::Stop)
    }
}

fn cargo_run(path: &Path, config_path: &Path) -> anyhow::Result<Command> {
    let flags = common::CargoFlags {
        otel: OTEL.load(std::sync::atomic::Ordering::Relaxed),
        fips: FIPS.load(std::sync::atomic::Ordering::Relaxed),
        release: RELEASE.load(std::sync::atomic::Ordering::Relaxed),
        locked: LOCKED.load(std::sync::atomic::Ordering::Relaxed),
    };
    common::cargo_command("run", path, config_path, flags)
}

fn cargo_run_loop(
    cargo_dir: &Path,
    config_path: &Path,
    signal_rx: &mpsc::Receiver<RunSignal>,
    event_tx: &mpsc::Sender<WatchEvent>,
) {
    'outer: loop {
        let mut child = match cargo_run(cargo_dir, config_path)
            .and_then(|mut cmd| cmd.spawn().context("failed to spawn cargo run"))
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("failed to spawn cargo run: {e}");
                _ = event_tx.send(WatchEvent::ChildExited);
                return;
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

        // Child exited on its own - notify the main thread so it can
        // break out of the event loop instead of blocking forever.
        _ = event_tx.send(WatchEvent::ChildExited);
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test: before the fix, when `cargo_run_loop` couldn't spawn a
    /// child, it blocked on `signal_rx.recv()` forever. Now it sends
    /// `ChildExited` and returns.
    #[test]
    fn runner_sends_child_exited_on_spawn_failure() {
        let (_signal_tx, signal_rx) = mpsc::channel::<RunSignal>();
        let (event_tx, event_rx) = mpsc::channel::<WatchEvent>();

        // Non-existent directory causes spawn() to fail.
        let cargo_dir = PathBuf::from("/nonexistent/cargo/dir");
        let config_path = PathBuf::from("/nonexistent/config.yml");

        let handle = std::thread::spawn(move || {
            cargo_run_loop(&cargo_dir, &config_path, &signal_rx, &event_tx);
        });

        let event = event_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("should receive ChildExited within timeout");
        assert!(matches!(event, WatchEvent::ChildExited));
        handle.join().expect("runner thread should not panic");
    }
}
