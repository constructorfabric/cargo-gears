use crate::common;
use crate::gears_parser::CargoTomlDependencies;
use anyhow::{Context, bail};
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct BuildParams {
    /// Resolved workspace root path.
    pub workspace_root: PathBuf,
    /// Resolved generated output directory.
    pub generated_dir: PathBuf,
    /// Resolved generated project name.
    pub generated_name: String,
    /// Resolved config file path.
    pub config_path: PathBuf,
    /// Resolved workspace dependencies.
    pub dependencies: CargoTomlDependencies,
    /// Effective otel flag (CLI override already applied).
    pub otel: bool,
    /// Effective FIPS flag.
    pub fips: bool,
    /// Effective release flag.
    pub release: bool,
    /// Whether to remove Cargo.lock before building.
    pub clean: bool,
    /// Print the resolved generation model without building.
    pub dry_run: bool,
}

impl BuildParams {
    pub fn run(&self) -> anyhow::Result<()> {
        if self.clean {
            common::remove_from_file_structure(
                &self.generated_dir,
                &self.generated_name,
                "Cargo.lock",
            )?;
        }

        let generated = common::generate_server_structure(
            &self.workspace_root,
            &self.generated_dir,
            &self.generated_name,
            &self.dependencies,
        )?;

        if self.dry_run {
            return generated.print_json();
        }

        let cargo_dir = common::generated_project_dir(&self.generated_dir, &self.generated_name);
        let status = common::cargo_command(
            "build",
            &cargo_dir,
            &self.config_path,
            self.otel,
            self.fips,
            self.release,
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
    use super::BuildParams;
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

        let resolved = ManifestSelection {
            manifest: PathBuf::from("Gears.toml"),
            app: Some("app".to_owned()),
            env: Some("dev".to_owned()),
        }
        .resolve(temp.path())
        .expect("manifest should resolve");

        BuildParams {
            workspace_root: resolved.workspace_root,
            generated_dir: resolved.generated_dir,
            generated_name: resolved.generated_name,
            config_path: resolved.config_path,
            dependencies: resolved.dependencies,
            otel: false,
            fips: false,
            release: false,
            clean: false,
            dry_run: true,
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
