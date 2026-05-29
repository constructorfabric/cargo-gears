use crate::common::{self, PathConfigParams};
use anyhow::{Context, bail};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

const DOCKERFILE_CONTENT: &str = include_str!("../../shared/Dockerfile");

#[derive(Debug, Eq, PartialEq)]
pub struct DeployParams {
    pub path_config: PathConfigParams,
    /// Tag to apply to the generated Docker image
    pub tag: Option<String>,
    /// Cargo manifest to build instead of generating a server project
    pub manifest: Option<PathBuf>,
    /// By default, builds in release mode. Use this for debug mode.
    pub debug: bool,
    /// Dockerfile path to use instead of the default
    pub dockerfile: Option<PathBuf>,
    /// Dockerfile ARG override in KEY=VALUE form. Can be repeated.
    pub args: Vec<DockerBuildArg>,
}

impl DeployParams {
    pub fn run(&self) -> anyhow::Result<()> {
        self.path_config
            .with_workspace_dir(|workspace_path, config_path| {
                let (manifest_path, artifact_name) = if let Some(manifest) = &self.manifest {
                    let manifest_path = resolve_manifest(manifest)?;
                    let artifact_name = manifest_package_name(&manifest_path)?;
                    (manifest_path, artifact_name)
                } else {
                    let project_name = common::resolve_generated_project_name(config_path, None)?;
                    let dependencies =
                        common::get_config(workspace_path, config_path)?.create_dependencies()?;
                    let generated_dir = workspace_path.join(common::DEFAULT_GENERATED_DIR);
                    common::generate_server_structure(
                        workspace_path,
                        &generated_dir,
                        &project_name,
                        &dependencies,
                    )?;
                    (
                        common::generated_project_dir(&generated_dir, &project_name)
                            .join("Cargo.toml"),
                        project_name,
                    )
                };

                let workspace_root = workspace_path
                    .canonicalize()
                    .context("can't canonicalize workspace root")?;
                ensure_dockerfile(&workspace_root)?;

                let manifest_arg =
                    path_inside_build_context(&manifest_path, &workspace_root, "manifest")?;
                let config_arg = path_inside_build_context(config_path, &workspace_root, "config")?;
                let config_ext = config_path
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .context("config must have a file extension")?;

                let mut command = Command::new("docker");
                command.arg("build");
                add_build_arg(&mut command, "BUILDER_MANIFEST", &manifest_arg);
                add_build_arg(
                    &mut command,
                    "BUILD_MODE",
                    if self.debug { "debug" } else { "release" },
                );
                add_build_arg(&mut command, "ARTIFACT_NAME", &artifact_name);
                add_build_arg(&mut command, "LOCAL_CONFIG_PATH", &config_arg);
                add_build_arg(&mut command, "CONFIG_EXT", config_ext);
                for arg in &self.args {
                    command.arg("--build-arg").arg(arg.to_string());
                }
                if let Some(tag) = &self.tag {
                    command.arg("--tag").arg(tag);
                } else {
                    let default_tag = format!("gears:{}", env!("CARGO_PKG_VERSION"));
                    command.arg("--tag").arg(default_tag);
                }
                if let Some(dockerfile) = &self.dockerfile {
                    let canonical_dockerfile = dockerfile.canonicalize().with_context(|| {
                        format!("dockerfile doesn't exists: {}", dockerfile.display())
                    })?;
                    command.arg("--file").arg(&canonical_dockerfile);
                }

                command.arg(".");
                command.current_dir(&workspace_root);

                let status = command.status().context("failed to run docker build")?;
                if !status.success() {
                    bail!("docker build exited with {status}");
                }

                Ok(())
            })
    }
}

fn ensure_dockerfile(workspace_root: &Path) -> anyhow::Result<()> {
    let dockerfile_path = workspace_root.join("Dockerfile");
    if dockerfile_path.exists() {
        return Ok(());
    }

    fs::write(&dockerfile_path, DOCKERFILE_CONTENT)
        .with_context(|| format!("failed to write {}", dockerfile_path.display()))
}

fn resolve_manifest(manifest: &Path) -> anyhow::Result<PathBuf> {
    if manifest.file_name().and_then(std::ffi::OsStr::to_str) != Some("Cargo.toml") {
        bail!("manifest must point to a Cargo.toml file");
    }

    manifest
        .canonicalize()
        .with_context(|| format!("can't canonicalize manifest {}", manifest.display()))
}

fn manifest_package_name(manifest_path: &Path) -> anyhow::Result<String> {
    let manifest = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;
    let manifest: toml::Value = toml::from_str(&manifest)
        .with_context(|| format!("failed to parse manifest {}", manifest_path.display()))?;

    manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .context("manifest must contain package.name")
}

fn path_inside_build_context(
    path: &Path,
    workspace_root: &Path,
    label: &str,
) -> anyhow::Result<PathBuf> {
    let path = path
        .canonicalize()
        .with_context(|| format!("can't canonicalize {label} path {}", path.display()))?;
    path.strip_prefix(workspace_root)
        .map(Path::to_path_buf)
        .with_context(|| {
            format!(
                "{label} path {} must be inside Docker build context {}",
                path.display(),
                workspace_root.display()
            )
        })
}

fn add_build_arg<T>(command: &mut Command, key: &str, value: T)
where
    T: AsRef<std::ffi::OsStr>,
{
    command
        .arg("--build-arg")
        .arg(format!("{key}={}", value.as_ref().to_string_lossy()));
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockerBuildArg {
    pub key: String,
    pub value: String,
}

impl FromStr for DockerBuildArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (key, value) = value
            .split_once('=')
            .ok_or_else(|| "expected KEY=VALUE".to_owned())?;
        if key.is_empty() {
            return Err("argument key cannot be empty".to_owned());
        }

        Ok(Self {
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }
}

impl fmt::Display for DockerBuildArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::{DockerBuildArg, manifest_package_name, resolve_manifest};
    use crate::module_parser::test_utils::TempDirExt;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn docker_build_arg_requires_key_value_pair() {
        assert_eq!(
            "BUILDER_FLAGS=--features demo"
                .parse::<DockerBuildArg>()
                .map(|arg| arg.to_string()),
            Ok("BUILDER_FLAGS=--features demo".to_owned())
        );
        assert!("BUILDER_FLAGS".parse::<DockerBuildArg>().is_err());
        assert!("=value".parse::<DockerBuildArg>().is_err());
    }

    #[test]
    fn resolve_manifest_requires_cargo_toml_filename() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        temp_dir.write("Cargo.toml", "");
        temp_dir.write("Other.toml", "");

        assert!(resolve_manifest(&temp_dir.path().join("Cargo.toml")).is_ok());
        assert!(resolve_manifest(&temp_dir.path().join("Other.toml")).is_err());

        Ok(())
    }

    #[test]
    fn manifest_package_name_reads_package_name() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        temp_dir.write(
            "Cargo.toml",
            r#"[package]
name = "demo-server"
version = "0.1.0"
edition = "2024"
"#,
        );

        let name = manifest_package_name(&temp_dir.path().join(Path::new("Cargo.toml")))?;

        assert_eq!(name, "demo-server");
        Ok(())
    }
}
