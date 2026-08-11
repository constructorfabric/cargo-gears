use crate::common;
use crate::generate::gear::remove_stale_workspace_members;
use crate::manifest::ManifestSelection;
use anyhow::Context;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct CleanParams {
    pub workspace_root: PathBuf,
    pub generated_dir: PathBuf,
    pub generated_name: String,
}

pub struct CleanParamsBuilder {
    workspace_path: Option<PathBuf>,
    manifest: PathBuf,
    app: Option<String>,
    env: Option<String>,
}

impl CleanParamsBuilder {
    #[must_use]
    pub const fn new(manifest: PathBuf) -> Self {
        Self {
            workspace_path: None,
            manifest,
            app: None,
            env: None,
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

    pub fn build(self) -> anyhow::Result<CleanParams> {
        let workspace_root = common::resolve_workspace_path(self.workspace_path.as_deref())?;
        let selection = ManifestSelection {
            manifest: self.manifest,
            app: self.app,
            env: self.env,
        };
        let target = selection.resolve_target(&workspace_root)?;

        Ok(CleanParams {
            workspace_root: target.workspace_root,
            generated_dir: target.generated_dir,
            generated_name: target.generated_name,
        })
    }
}

impl CleanParams {
    pub fn run(&self) -> anyhow::Result<()> {
        // Run `cargo clean` first to remove build artifacts.
        // If it fails due to a broken workspace (e.g. missing generated
        // member), warn and continue -- fixing that is our job.
        match common::cargo_cmd() {
            Ok(mut cmd) => {
                cmd.arg("clean").current_dir(&self.workspace_root);
                let output = cmd.output().context("failed to spawn cargo clean")?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if Self::is_workspace_error(&stderr) {
                        eprintln!("warning: cargo clean failed (broken workspace), continuing");
                    } else {
                        anyhow::bail!(
                            "cargo clean exited with {}: {}",
                            output.status,
                            stderr.trim()
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("warning: {e}, skipping cargo clean");
            }
        }

        let project_dir = common::generated_project_dir(&self.generated_dir, &self.generated_name);

        // 1. Delete the generated project directory, so that the
        //    stale-member cleanup sees it as missing and removes the entry.
        if project_dir.exists() {
            fs::remove_dir_all(&project_dir).with_context(|| {
                format!(
                    "failed to remove generated project {}",
                    project_dir.display()
                )
            })?;
            println!("removed {}", project_dir.display());
        }

        // 2. Remove the (now-stale) member from workspace Cargo.toml.
        remove_stale_workspace_members(&self.workspace_root, &self.generated_dir)?;

        // 3. Remove the generated dir if it is now empty.
        if self.generated_dir.exists()
            && fs::read_dir(&self.generated_dir).is_ok_and(|mut d| d.next().is_none())
        {
            fs::remove_dir(&self.generated_dir).with_context(|| {
                format!(
                    "failed to remove empty generated dir {}",
                    self.generated_dir.display()
                )
            })?;
        }

        // 4. Run `cargo clean` to remove build artifacts. The workspace
        //    is now consistent, so this should succeed.
        let mut cmd = common::cargo_cmd()?;
        cmd.arg("clean").current_dir(&self.workspace_root);
        let status = cmd.status().context("failed to run cargo clean")?;
        if !status.success() {
            anyhow::bail!("cargo clean exited with {status}");
        }

        Ok(())
    }

    /// Returns `true` when the `cargo clean` stderr looks like a broken
    /// workspace (missing member, unreadable manifest, etc.) -- exactly
    /// the kind of state `gears clean` is meant to fix.
    fn is_workspace_error(stderr: &str) -> bool {
        let markers = [
            "failed to load manifest",
            "failed to read",
            "workspace member",
            "No such file or directory",
        ];
        markers.iter().any(|m| stderr.contains(m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ManifestSelection;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_workspace(temp: &TempDir) {
        fs::write(
            temp.path().join("Gears.toml"),
            r#"
[apps.app.dev]
config = "app-dev.yml"
gears = []
"#,
        )
        .unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\".gears/app-dev\"]\nresolver = \"3\"\n",
        )
        .unwrap();
        fs::create_dir_all(temp.path().join("config")).unwrap();
        fs::write(temp.path().join("config/app-dev.yml"), "server: {}\n").unwrap();
        // Create the generated project directory
        let project_dir = temp.path().join(".gears/app-dev");
        fs::create_dir_all(project_dir.join("src")).unwrap();
        fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"app-dev\"\n",
        )
        .unwrap();
        fs::write(project_dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    }

    #[test]
    fn clean_removes_generated_project_and_workspace_member() {
        let temp = TempDir::new().unwrap();
        setup_workspace(&temp);

        let project_dir = temp.path().join(".gears/app-dev");
        assert!(project_dir.exists());

        let target = ManifestSelection {
            manifest: PathBuf::from("Gears.toml"),
            app: Some("app".to_owned()),
            env: Some("dev".to_owned()),
        }
        .resolve_target(temp.path())
        .unwrap();

        let params = CleanParams {
            workspace_root: target.workspace_root,
            generated_dir: target.generated_dir,
            generated_name: target.generated_name,
        };
        params.run().unwrap();

        // Generated project should be gone
        assert!(!project_dir.exists());
        // Generated dir should be gone (was empty after removal)
        assert!(!temp.path().join(".gears").exists());
        // Workspace member should be removed from Cargo.toml
        let cargo_toml = fs::read_to_string(temp.path().join("Cargo.toml")).unwrap();
        assert!(
            !cargo_toml.contains(".gears/app-dev"),
            "workspace member should be removed"
        );
    }

    #[test]
    fn clean_is_idempotent_when_already_clean() {
        let temp = TempDir::new().unwrap();
        setup_workspace(&temp);

        let target = ManifestSelection {
            manifest: PathBuf::from("Gears.toml"),
            app: Some("app".to_owned()),
            env: Some("dev".to_owned()),
        }
        .resolve_target(temp.path())
        .unwrap();

        let params = CleanParams {
            workspace_root: target.workspace_root,
            generated_dir: target.generated_dir,
            generated_name: target.generated_name,
        };

        // First clean
        params.run().unwrap();
        // Second clean should not error
        params.run().unwrap();
    }

    #[test]
    fn is_workspace_error_detects_missing_manifest() {
        let stderr = "error: failed to load manifest for workspace member \
                       `/tmp/foo/.gears/app-dev`\n\nCaused by:\n  \
                       failed to read `/tmp/foo/.gears/app-dev/Cargo.toml`\n\n\
                       Caused by:\n  No such file or directory (os error 2)\n";
        assert!(CleanParams::is_workspace_error(stderr));
    }

    #[test]
    fn is_workspace_error_rejects_unrelated_failure() {
        let stderr = "error: Permission denied (os error 13)\n";
        assert!(!CleanParams::is_workspace_error(stderr));
    }
}
