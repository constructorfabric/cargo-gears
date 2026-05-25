use clap::Args;
use cyberware_cli_core::app_config;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

pub use cyberware_cli_core::app_config::DbEngineCfg;
pub use cyberware_cli_core::common::{OutputFormat, Registry};

#[derive(Args)]
pub struct PathConfigArgs {
    /// Path to the module workspace root
    #[arg(short = 'p', long, value_parser = cyberware_cli_core::common::parse_and_chdir)]
    pub path: Option<PathBuf>,
    /// Path to the config file
    #[arg(short = 'c', long)]
    pub config: PathBuf,
}

impl From<PathConfigArgs> for cyberware_cli_core::common::PathConfigArgs {
    fn from(args: PathConfigArgs) -> Self {
        Self {
            path: args.path,
            config: args.config,
        }
    }
}

#[derive(Args)]
pub struct BuildRunArgs {
    #[command(flatten)]
    pub path_config: PathConfigArgs,
    /// Use OpenTelemetry tracing
    #[arg(long)]
    pub otel: bool,
    /// Enable FIPS mode
    #[arg(long)]
    pub fips: bool,
    /// Build/run in release mode
    #[arg(short = 'r', long)]
    pub release: bool,
    /// Remove Cargo.lock at the start of the execution
    #[arg(long)]
    pub clean: bool,
    /// Override the generated server and binary name
    #[arg(long)]
    pub name: Option<String>,
}

impl From<BuildRunArgs> for cyberware_cli_core::common::BuildRunArgs {
    fn from(args: BuildRunArgs) -> Self {
        Self {
            path_config: args.path_config.into(),
            otel: args.otel,
            fips: args.fips,
            release: args.release,
            clean: args.clean,
            name: args.name,
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
