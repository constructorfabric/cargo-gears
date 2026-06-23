use cargo_gears_core::app_config;
use clap::{ArgAction, Args};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

pub use cargo_gears_core::app_config::DbEngineCfg;
pub use cargo_gears_core::common::{OutputFormat, Registry};

#[derive(Args)]
pub struct WorkspacePath {
    /// Path to the module workspace root
    #[arg(short = 'p', long, value_parser = cargo_gears_core::common::parse_path)]
    pub(crate) path: Option<PathBuf>,
}

#[derive(Args)]
pub struct ManifestPath {
    /// Path to the Gears manifest file
    #[arg(long, default_value = cargo_gears_core::manifest::DEFAULT_MANIFEST_FILE)]
    pub(crate) manifest: PathBuf,
}

#[derive(Args)]
pub struct PathConfigArgs {
    #[command(flatten)]
    pub workspace: WorkspacePath,
    /// Path to the config file
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,
}

impl From<PathConfigArgs> for cargo_gears_core::common::PathConfigParams {
    fn from(args: PathConfigArgs) -> Self {
        Self {
            path: args.workspace.path,
            config: args.config,
        }
    }
}

#[derive(Args)]
pub struct BuildRunArgs {
    #[command(flatten)]
    pub(crate) workspace: WorkspacePath,
    #[command(flatten)]
    pub(crate) manifest: ManifestTargetArgs,
    /// Use OpenTelemetry tracing
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_otel")]
    pub(crate) otel: bool,
    /// Disable OpenTelemetry tracing
    #[arg(long = "no-otel", action = ArgAction::SetTrue, conflicts_with = "otel")]
    pub(crate) no_otel: bool,
    /// Enable FIPS mode
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_fips")]
    pub(crate) fips: bool,
    /// Disable FIPS mode
    #[arg(long = "no-fips", action = ArgAction::SetTrue, conflicts_with = "fips")]
    pub(crate) no_fips: bool,
    /// Build/run in release mode
    #[arg(short = 'r', long, action = ArgAction::SetTrue, conflicts_with = "no_release")]
    pub(crate) release: bool,
    /// Build/run without release mode
    #[arg(long = "no-release", action = ArgAction::SetTrue, conflicts_with = "release")]
    pub(crate) no_release: bool,
    /// Remove Cargo.lock at the start of the execution
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_clean")]
    pub(crate) clean: bool,
    /// Do not remove Cargo.lock at the start of the execution
    #[arg(long = "no-clean", action = ArgAction::SetTrue, conflicts_with = "clean")]
    pub(crate) no_clean: bool,
    /// Print the resolved generation model without building or running
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Override the generated server and binary name
    #[arg(long)]
    pub(crate) name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::BuildRunArgs;
    use cargo_gears_core::build::BuildParamsBuilder;
    use cargo_gears_core::run::RunParamsBuilder;
    use clap::Parser;
    use std::fs;
    use tempfile::TempDir;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        br: BuildRunArgs,
    }

    fn parse(temp: &TempDir, extra: &[&str]) -> BuildRunArgs {
        let p = temp.path().to_str().expect("temp path should be UTF-8");
        let mut args = vec!["test", "-p", p, "--app", "app", "--env", "dev"];
        args.extend(extra);
        TestCli::try_parse_from(args).expect("should parse").br
    }

    fn write_workspace(temp: &TempDir, manifest: &str) {
        fs::write(temp.path().join("Gears.toml"), manifest).expect("write manifest");
        fs::create_dir_all(temp.path().join("config")).expect("create config dir");
        fs::write(temp.path().join("config/app-dev.yml"), "server: {}\n").expect("write config");
    }

    #[test]
    fn cli_overrides_take_precedence_over_manifest() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.run]\n\
             otel = true\n\
             fips = true\n\
             [apps.app.dev.build]\n\
             profile = \"release\"\n\
             clean = true\n",
        );

        let args = parse(
            &temp,
            &["--no-otel", "--no-fips", "--no-release", "--no-clean"],
        );

        let resolved = BuildParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .build()
            .expect("resolve");

        assert!(!resolved.build_run_args.otel());
        assert!(!resolved.build_run_args.fips());
        assert!(!resolved.build_run_args.release());
        assert!(!resolved.build_run_args.clean());
    }

    #[test]
    fn defaults_to_manifest_policy_when_no_cli_override() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.run]\n\
             otel = true\n\
             fips = true\n\
             [apps.app.dev.build]\n\
             profile = \"release\"\n\
             clean = true\n",
        );

        let args = parse(&temp, &[]);

        let resolved = BuildParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .build()
            .expect("resolve");

        assert!(resolved.build_run_args.otel());
        assert!(resolved.build_run_args.fips());
        assert!(resolved.build_run_args.release());
        assert!(resolved.build_run_args.clean());
    }

    #[test]
    fn clean_defaults_to_release_when_manifest_omits_clean() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.build]\n\
             profile = \"release\"\n",
        );

        let args = parse(&temp, &[]);

        let resolved = BuildParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .build()
            .expect("resolve");

        assert!(resolved.build_run_args.release());
        assert!(resolved.build_run_args.clean());
    }

    #[test]
    fn clean_defaults_to_false_for_debug_builds() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n",
        );

        let args = parse(&temp, &[]);

        let resolved = BuildParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .build()
            .expect("resolve");

        assert!(!resolved.build_run_args.release());
        assert!(!resolved.build_run_args.clean());
    }

    #[test]
    fn run_builder_cli_overrides_take_precedence_over_manifest() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.run]\n\
             otel = true\n\
             fips = true\n\
             [apps.app.dev.run.watch]\n\
             enabled = true\n\
             [apps.app.dev.build]\n\
             profile = \"release\"\n\
             clean = true\n",
        );

        let args = parse(
            &temp,
            &["--no-otel", "--no-fips", "--no-release", "--no-clean"],
        );

        let resolved = RunParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .watch(None)
            .no_watch(None)
            .build()
            .expect("resolve");

        assert!(!resolved.build_run_args.otel());
        assert!(!resolved.build_run_args.fips());
        assert!(!resolved.build_run_args.release());
        assert!(!resolved.build_run_args.clean());
    }

    #[test]
    fn run_builder_defaults_to_manifest_policy_when_no_cli_override() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.run]\n\
             otel = true\n\
             fips = true\n\
             [apps.app.dev.run.watch]\n\
             enabled = true\n\
             [apps.app.dev.build]\n\
             profile = \"release\"\n\
             clean = true\n",
        );

        let args = parse(&temp, &[]);

        let resolved = RunParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .watch(None)
            .no_watch(None)
            .build()
            .expect("resolve");

        assert!(resolved.build_run_args.otel());
        assert!(resolved.build_run_args.fips());
        assert!(resolved.build_run_args.release());
        assert!(resolved.build_run_args.clean());
        assert!(resolved.watch);
    }

    #[test]
    fn cli_name_override_works_for_build() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.build]\n\
             name = \"demo-server\"\n",
        );

        let args = parse(&temp, &["--name", "custom-name"]);

        let resolved = BuildParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .build()
            .expect("resolve");

        assert_eq!(resolved.build_run_args.generated_name(), "custom-name");
    }

    #[test]
    fn cli_name_override_works_for_run() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.build]\n\
             name = \"demo-server\"\n",
        );

        let args = parse(&temp, &["--name", "custom-name"]);

        let resolved = RunParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .watch(None)
            .no_watch(None)
            .build()
            .expect("resolve");

        assert_eq!(resolved.build_run_args.generated_name(), "custom-name");
    }

    #[test]
    fn cli_name_override_defaults_to_manifest_when_not_provided() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            &temp,
            "[apps.app.dev]\n\
             config = \"app-dev.yml\"\n\
             modules = []\n\
             [apps.app.dev.build]\n\
             name = \"demo-server\"\n",
        );

        let args = parse(&temp, &[]);

        let resolved = BuildParamsBuilder::new(args.manifest.manifest_path.manifest)
            .workspace_path(args.workspace.path)
            .app(args.manifest.app)
            .env(args.manifest.env)
            .name(args.name)
            .otel(args.otel.then_some(true))
            .no_otel(args.no_otel.then_some(true))
            .fips(args.fips.then_some(true))
            .no_fips(args.no_fips.then_some(true))
            .release(args.release.then_some(true))
            .no_release(args.no_release.then_some(true))
            .clean(args.clean.then_some(true))
            .no_clean(args.no_clean.then_some(true))
            .dry_run(args.dry_run)
            .build()
            .expect("resolve");

        assert_eq!(resolved.build_run_args.generated_name(), "demo-server");
    }
}

#[must_use]
#[derive(Args)]
pub struct ManifestTargetArgs {
    #[command(flatten)]
    pub(crate) manifest_path: ManifestPath,
    /// Manifest app to select (inferred from manifest if omitted)
    #[arg(long)]
    pub(crate) app: Option<String>,
    /// Manifest environment to select (inferred from manifest if omitted)
    #[arg(long)]
    pub(crate) env: Option<String>,
}

impl ManifestTargetArgs {
    #[must_use]
    pub fn into_selection(self) -> cargo_gears_core::manifest::ManifestSelection {
        cargo_gears_core::manifest::ManifestSelection {
            manifest: self.manifest_path.manifest,
            app: self.app,
            env: self.env,
        }
    }
}

#[derive(Clone, Args)]
pub struct DbConnConfig {
    /// Explicit database engine for this connection.
    #[arg(long, value_enum)]
    pub engine: Option<DbEngineCfg>,
    /// DSN-style (full, valid). Optional: can be absent and rely on fields.
    #[arg(long)]
    pub dsn: Option<String>,
    /// Field-based style; any of these override DSN parts when present.
    #[arg(long)]
    pub host: Option<String>,
    #[arg(long)]
    pub port: Option<u16>,
    #[arg(long)]
    pub user: Option<String>,
    /// Literal password or `${VAR}` for env expansion.
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long)]
    pub dbname: Option<String>,
    #[arg(long = "params", value_parser = parse_params_map)]
    pub params: Option<BTreeMap<String, String>>,
    /// `SQLite` file-based helpers.
    #[arg(long = "sqlite-file")]
    pub file: Option<String>,
    #[arg(id = "db_path", long = "sqlite-path", value_name = "PATH")]
    pub path: Option<PathBuf>,
    /// Connection pool overrides.
    #[command(flatten)]
    pub pool: Option<PoolCfg>,
    /// Reference to a global server by name.
    #[arg(long)]
    pub server: Option<String>,
}

impl From<DbConnConfig> for app_config::DbConnConfig {
    fn from(conn: DbConnConfig) -> Self {
        Self {
            engine: conn.engine,
            dsn: conn.dsn,
            host: conn.host,
            port: conn.port,
            user: conn.user,
            password: conn.password,
            dbname: conn.dbname,
            params: conn.params,
            file: conn.file,
            path: conn.path,
            pool: conn.pool.map(Into::into),
            server: conn.server,
        }
    }
}

#[derive(Clone, Args)]
pub struct PoolCfg {
    #[arg(long = "pool-max-conns")]
    pub max_conns: Option<u32>,
    #[arg(long = "pool-min-conns")]
    pub min_conns: Option<u32>,
    #[arg(long = "pool-acquire-timeout-secs", value_parser = parse_duration_secs)]
    pub acquire_timeout: Option<Duration>,
    #[arg(long = "pool-idle-timeout-secs", value_parser = parse_duration_secs)]
    pub idle_timeout: Option<Duration>,
    #[arg(long = "pool-max-lifetime-secs", value_parser = parse_duration_secs)]
    pub max_lifetime: Option<Duration>,
    #[arg(long = "pool-test-before-acquire")]
    pub test_before_acquire: Option<bool>,
}

impl From<PoolCfg> for app_config::PoolCfg {
    fn from(pool: PoolCfg) -> Self {
        Self {
            max_conns: pool.max_conns,
            min_conns: pool.min_conns,
            acquire_timeout: pool.acquire_timeout,
            idle_timeout: pool.idle_timeout,
            max_lifetime: pool.max_lifetime,
            test_before_acquire: pool.test_before_acquire,
        }
    }
}

fn parse_params_map(raw: &str) -> Result<BTreeMap<String, String>, String> {
    let mut params = BTreeMap::new();
    for pair in raw.split(',') {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| format!("invalid key=value pair '{pair}'"))?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() {
            return Err(format!("invalid key=value pair '{pair}'"));
        }
        params.insert(key.to_owned(), value.to_owned());
    }

    if params.is_empty() {
        return Err("params cannot be empty".to_owned());
    }

    Ok(params)
}

fn parse_duration_secs(raw: &str) -> Result<Duration, String> {
    raw.parse::<u64>()
        .map(Duration::from_secs)
        .map_err(|_| format!("invalid duration seconds '{raw}'"))
}
