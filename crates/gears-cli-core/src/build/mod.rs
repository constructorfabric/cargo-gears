use crate::common::{self, BuildRunArgs};
use anyhow::{Context, bail};

#[derive(Debug, Eq, PartialEq)]
pub struct BuildArgs {
    pub build_run_args: BuildRunArgs,
}

impl BuildArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_path = common::resolve_workspace_path(self.build_run_args.path.as_deref())?;
        let resolved = self.build_run_args.manifest.resolve(&workspace_path)?;

        self.build_run_args.clean_build(&resolved)?;

        let generated = common::generate_server_structure(
            &resolved.workspace_root,
            &resolved.generated_dir,
            &resolved.generated_name,
            &resolved.dependencies,
        )?;

        if self.build_run_args.dry_run {
            return generated.print_json();
        }

        let cargo_dir =
            common::generated_project_dir(&resolved.generated_dir, &resolved.generated_name);
        let status = common::cargo_command(
            "build",
            &cargo_dir,
            &resolved.config_path,
            self.build_run_args.otel_enabled(&resolved),
            self.build_run_args.fips_enabled(&resolved),
            self.build_run_args.release_build(&resolved),
        )?
        .status()
        .context("failed to run cargo build")?;

        if !status.success() {
            bail!("cargo build exited with {status}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BuildArgs;
    use crate::common::BuildRunArgs;
    use crate::manifest::ManifestSelection;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn dry_run_generates_files_under_manifest_generated_dir_and_name() {
        let temp = TempDir::new().expect("temp dir should be created");
        let manifest_path = temp.path().join("Gears.toml");
        fs::write(
            &manifest_path,
            r#"
[workspace]
generated-dir = "generated-output"

[apps.app.dev]
config = "app-dev.yml"
modules = [{ source = "remote", name = "module", package = "cf-module", version = "1.2" }]

[apps.app.dev.build]
name = "demo-server"
"#,
        )
        .expect("manifest should be written");
        fs::create_dir_all(temp.path().join("config")).expect("config dir should be created");
        fs::write(temp.path().join("config/app-dev.yml"), "server: {}\n")
            .expect("config should be written");

        BuildArgs {
            build_run_args: BuildRunArgs {
                path: Some(temp.path().to_path_buf()),
                manifest: ManifestSelection {
                    manifest: PathBuf::from("Gears.toml"),
                    app: Some("app".to_owned()),
                    env: Some("dev".to_owned()),
                },
                otel: None,
                fips: None,
                release: None,
                clean: None,
                dry_run: true,
                name: None,
            },
        }
        .run()
        .expect("build dry-run should generate server files");

        let generated_project = temp.path().join("generated-output/demo-server");
        assert!(generated_project.join("Cargo.toml").is_file());
        assert!(generated_project.join(".cargo/config.toml").is_file());
        assert!(generated_project.join("src/main.rs").is_file());
        assert!(!temp.path().join("demo-server/Cargo.toml").exists());
        assert!(!temp.path().join(".gears/demo-server/Cargo.toml").exists());
    }
}
