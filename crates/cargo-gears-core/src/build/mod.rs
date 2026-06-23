use crate::common::{self, BuildRunParams};
use crate::manifest::{BuildProfile, ManifestSelection};
use anyhow::{Context, bail};
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct BuildParams {
    /// Shared build/run parameters.
    pub build_run_args: BuildRunParams,
}

/// Builder for constructing `BuildParams` with manifest resolution and CLI overrides.
pub struct BuildParamsBuilder {
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
}

impl BuildParamsBuilder {
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

    pub fn build(self) -> anyhow::Result<BuildParams> {
        let workspace_root = common::resolve_workspace_path(self.workspace_path.as_deref())?;
        let manifest_selection = ManifestSelection {
            manifest: self.manifest,
            app: self.app,
            env: self.env,
        };
        let resolved = manifest_selection.resolve(&workspace_root)?;

        let otel = common::ordered_bool(self.otel, self.no_otel).unwrap_or(resolved.run.otel);
        let fips = common::ordered_bool(self.fips, self.no_fips).unwrap_or(resolved.run.fips);
        let release = common::ordered_bool(self.release, self.no_release).unwrap_or(matches!(
            resolved.build.profile,
            Some(BuildProfile::Release)
        ));
        let clean = common::ordered_bool(self.clean, self.no_clean)
            .unwrap_or_else(|| resolved.build.clean.unwrap_or(release));

        Ok(BuildParams {
            build_run_args: BuildRunParams {
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
        })
    }
}

impl BuildParams {
    pub fn run(&self) -> anyhow::Result<()> {
        if self.build_run_args.clean {
            common::remove_from_file_structure(
                &self.build_run_args.generated_dir,
                &self.build_run_args.generated_name,
                "Cargo.lock",
            )?;
        }

        let generated = common::generate_server_structure(
            &self.build_run_args.workspace_root,
            &self.build_run_args.generated_dir,
            &self.build_run_args.generated_name,
            &self.build_run_args.dependencies,
        )?;

        if self.build_run_args.dry_run {
            return generated.print_json();
        }

        let cargo_dir = common::generated_project_dir(
            &self.build_run_args.generated_dir,
            &self.build_run_args.generated_name,
        );
        let status = common::cargo_command(
            "build",
            &cargo_dir,
            &self.build_run_args.config_path,
            self.build_run_args.otel,
            self.build_run_args.fips,
            self.build_run_args.release,
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
    use crate::common::BuildRunParams;
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
            build_run_args: BuildRunParams {
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
