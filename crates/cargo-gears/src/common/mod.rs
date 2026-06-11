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
    pub path: Option<PathBuf>,
}

#[derive(Args)]
pub struct ManifestPath {
    /// Path to the Gears manifest file
    #[arg(long, default_value = cargo_gears_core::manifest::DEFAULT_MANIFEST_FILE)]
    pub manifest: PathBuf,
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
    pub workspace: WorkspacePath,
    #[command(flatten)]
    pub manifest: ManifestTargetArgs,
    /// Use OpenTelemetry tracing
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_otel")]
    pub otel: bool,
    /// Disable OpenTelemetry tracing
    #[arg(long = "no-otel", action = ArgAction::SetTrue, conflicts_with = "otel")]
    pub no_otel: bool,
    /// Enable FIPS mode
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_fips")]
    pub fips: bool,
    /// Disable FIPS mode
    #[arg(long = "no-fips", action = ArgAction::SetTrue, conflicts_with = "fips")]
    pub no_fips: bool,
    /// Build/run in release mode
    #[arg(short = 'r', long, action = ArgAction::SetTrue, conflicts_with = "no_release")]
    pub release: bool,
    /// Build/run without release mode
    #[arg(long = "no-release", action = ArgAction::SetTrue, conflicts_with = "release")]
    pub no_release: bool,
    /// Remove Cargo.lock at the start of the execution
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_clean")]
    pub clean: bool,
    /// Do not remove Cargo.lock at the start of the execution
    #[arg(long = "no-clean", action = ArgAction::SetTrue, conflicts_with = "clean")]
    pub no_clean: bool,
    /// Print the resolved generation model without building or running
    #[arg(long)]
    pub dry_run: bool,
    /// Override the generated server and binary name
    #[arg(long)]
    pub name: Option<String>,
}

/// Fully-resolved build/run parameters after manifest + CLI override merge.
pub struct ResolvedBuildRun {
    pub workspace_root: std::path::PathBuf,
    pub generated_dir: std::path::PathBuf,
    pub generated_name: String,
    pub config_path: std::path::PathBuf,
    pub manifest_path: std::path::PathBuf,
    pub dependencies: cargo_gears_core::gears_parser::CargoTomlDependencies,
    pub otel: bool,
    pub fips: bool,
    pub release: bool,
    pub clean: bool,
    pub dry_run: bool,
    pub watch_policy: cargo_gears_core::manifest::WatchPolicy,
}

impl BuildRunArgs {
    /// Resolve the manifest and merge CLI overrides into final values.
    ///
    /// Takes `&self` so it can be called repeatedly (e.g. in a watch loop).
    pub fn resolve(&self) -> anyhow::Result<ResolvedBuildRun> {
        let workspace_path =
            cargo_gears_core::common::resolve_workspace_path(self.workspace.path.as_deref())?;
        let manifest_selection = cargo_gears_core::manifest::ManifestSelection {
            manifest: self.manifest.manifest_path.manifest.clone(),
            app: self.manifest.app.clone(),
            env: self.manifest.env.clone(),
        };
        let resolved = manifest_selection.resolve(&workspace_path)?;

        let otel = ordered_bool(self.otel, self.no_otel).unwrap_or(resolved.run.otel);
        let fips = ordered_bool(self.fips, self.no_fips).unwrap_or(resolved.run.fips);
        let release = ordered_bool(self.release, self.no_release).unwrap_or(matches!(
            resolved.build.profile,
            Some(cargo_gears_core::manifest::BuildProfile::Release)
        ));
        let clean = ordered_bool(self.clean, self.no_clean)
            .unwrap_or_else(|| resolved.build.clean.unwrap_or(release));

        Ok(ResolvedBuildRun {
            workspace_root: resolved.workspace_root,
            generated_dir: resolved.generated_dir,
            generated_name: resolved.generated_name,
            config_path: resolved.config_path,
            manifest_path: resolved.manifest_path,
            dependencies: resolved.dependencies,
            otel,
            fips,
            release,
            clean,
            dry_run: self.dry_run,
            watch_policy: resolved.run.watch,
        })
    }
}

#[must_use]
pub const fn ordered_bool(positive: bool, negative: bool) -> Option<bool> {
    match (positive, negative) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}

#[derive(Args)]
pub struct ManifestTargetArgs {
    #[command(flatten)]
    pub manifest_path: ManifestPath,
    /// Manifest app to select (inferred from manifest if omitted)
    #[arg(long)]
    pub app: Option<String>,
    /// Manifest environment to select (inferred from manifest if omitted)
    #[arg(long)]
    pub env: Option<String>,
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
