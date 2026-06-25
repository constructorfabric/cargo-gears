use crate::gears_parser::ConfigModuleMetadata;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::Duration;

/// Main application configuration with strongly-typed global sections
/// and a flexible per-module configuration bag.
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct AppConfig {
    /// Core server configuration.
    pub server: ServerConfig,
    /// Typed database configuration (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<GlobalDatabaseConfig>,
    /// Logging configuration.
    #[serde(default = "default_logging_config")]
    pub logging: LoggingConfig,
    /// OpenTelemetry configuration (resource, tracing, metrics).
    #[serde(default)]
    pub opentelemetry: OpenTelemetryConfig,
    /// Directory containing per-gear YAML files (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gears_dir: Option<String>,
    /// Per-gear configuration bag: `gear_name` -> gear config.
    #[serde(default)]
    pub gears: BTreeMap<String, ModuleConfig>,
    /// Per-vendor configuration bag: `vendor_name` → arbitrary JSON/YAML value.
    /// Allows vendors to add their own typed configuration sections.
    #[serde(default)]
    pub vendor: VendorConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: None,
            logging: default_logging_config(),
            opentelemetry: OpenTelemetryConfig::default(),
            gears_dir: None,
            gears: BTreeMap::new(),
            vendor: VendorConfig::default(),
        }
    }
}

/// Core server configuration.
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ServerConfig {
    /// Server home directory.
    #[serde(default = "default_home_dir")]
    pub home_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            home_dir: default_home_dir(),
        }
    }
}

/// Logging configuration - maps subsystem names to their logging settings
pub type LoggingConfig = BTreeMap<String, Section>;

/// Create a default logging configuration
#[must_use]
pub fn default_logging_config() -> LoggingConfig {
    let mut logging = BTreeMap::new();
    logging.insert(
        "default".to_owned(),
        Section {
            console_level: Some(tracing::Level::INFO),
            section_file: Some(SectionFile {
                file: "logs/cf-gears.log".to_owned(),
                file_level: Some(tracing::Level::DEBUG),
            }),
            console_format: ConsoleFormat::default(),
            max_age_days: Some(7),
            max_backups: Some(3),
            max_size_mb: Some(100),
        },
    );
    logging
}

mod optional_level_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use tracing::Level;

    #[allow(clippy::ref_option, clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(level: &Option<Level>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match level {
            Some(l) => serializer.serialize_str(l.as_str()),
            None => serializer.serialize_str("off"),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Level>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "trace" => Ok(Some(Level::TRACE)),
            "debug" => Ok(Some(Level::DEBUG)),
            "info" => Ok(Some(Level::INFO)),
            "warn" => Ok(Some(Level::WARN)),
            "error" => Ok(Some(Level::ERROR)),
            "off" | "none" => Ok(None),
            _ => Err(serde::de::Error::custom(format!("invalid level: {s}"))),
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn default() -> Option<Level> {
        Some(Level::INFO)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Section {
    #[serde(default)]
    pub console_format: ConsoleFormat,
    #[serde(
        default = "optional_level_serde::default",
        with = "optional_level_serde",
    )]
    pub console_level: Option<tracing::Level>,
    #[serde(flatten)]
    pub section_file: Option<SectionFile>,
    pub max_age_days: Option<u32>, // Not implemented yet
    #[serde(default)]
    pub max_backups: Option<usize>, // How many files to keep
    #[serde(default)]
    pub max_size_mb: Option<u64>, // Max size of the file in MB
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, Clone)]
pub struct SectionFile {
    pub file: String,
    #[serde(
        default = "optional_level_serde::default",
        with = "optional_level_serde",
    )]
    pub file_level: Option<tracing::Level>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleFormat {
    /// Human-readable text output (default).
    #[default]
    Text,
    /// Structured JSON output (useful for container log collectors).
    Json,
}

/// Per-module configuration: database, config bag, runtime, and Cargo metadata
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ModuleConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<DbConnConfig>,
    #[serde(default = "default_module_config")]
    pub config: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<ModuleRuntime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ConfigModuleMetadata>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            database: None,
            config: default_module_config(),
            runtime: None,
            metadata: None,
        }
    }
}

/// Runtime configuration for a module (local vs out-of-process)
#[derive(Clone, Deserialize, Serialize, Default, schemars::JsonSchema)]
pub struct ModuleRuntime {
    #[serde(default, rename = "type")]
    pub mod_type: RuntimeKind,
    /// Execution configuration for `OoP` gears.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionConfig>,
}

/// Execution configuration for out-of-process gears
#[derive(Clone, Deserialize, Serialize, Default, schemars::JsonSchema)]
pub struct ExecutionConfig {
    /// Path to the executable. Supports absolute paths or `~` expansion.
    pub executable_path: String,
    /// Command-line arguments to pass to the executable.
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory for the process (optional, defaults to current dir).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    /// Environment variables to set for the process.
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

/// Module runtime kind
#[derive(Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
    #[default]
    Local,
    Oop,
}

fn default_home_dir() -> PathBuf {
    PathBuf::from(".gears")
}

fn default_module_config() -> Value {
    Value::Object(Map::default())
}

/// Global database configuration with server-based DBs
#[derive(Clone, Deserialize, Serialize, Default, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GlobalDatabaseConfig {
    /// Server-based DBs (postgres/mysql/sqlite/etc.), keyed by server name.
    #[serde(default)]
    pub servers: BTreeMap<String, DbConnConfig>,
    /// Optional dev-only flag to auto-provision DB/schema when missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_provision: Option<bool>,
}

/// Reusable DB connection config for both global servers and gears
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DbConnConfig {
    /// Explicit database engine for this connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<DbEngineCfg>,
    /// DSN-style (full, valid). Optional: can be absent and rely on fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dsn: Option<String>,
    /// Field-based style; any of these override DSN parts when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Literal password or `${VAR}` for env expansion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dbname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<BTreeMap<String, String>>,
    /// `SQLite` file-based helpers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Connection pool overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pool: Option<PoolCfg>,
    /// Reference to a global server by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
}

/// Serializable engine selector for configuration
#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum DbEngineCfg {
    Postgres,
    Mysql,
    Sqlite,
}

/// Connection pool configuration
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PoolCfg {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_conns: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_conns: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquire_timeout: Option<Duration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_timeout: Option<Duration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_lifetime: Option<Duration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_before_acquire: Option<bool>,
}

impl DbConnConfig {
    #[must_use]
    pub fn has_any_value(&self) -> bool {
        self.engine.is_some()
            || self.dsn.is_some()
            || self.host.is_some()
            || self.port.is_some()
            || self.user.is_some()
            || self.password.is_some()
            || self.dbname.is_some()
            || self
                .params
                .as_ref()
                .is_some_and(|params| !params.is_empty())
            || self.file.is_some()
            || self.path.is_some()
            || self.server.is_some()
            || self.pool.as_ref().is_some_and(PoolCfg::has_any_value)
    }

    pub fn apply_patch(&mut self, patch: Self) {
        if let Some(engine) = patch.engine {
            self.engine = Some(engine);
        }
        if let Some(dsn) = patch.dsn {
            self.dsn = Some(dsn);
        }
        if let Some(host) = patch.host {
            self.host = Some(host);
        }
        if let Some(port) = patch.port {
            self.port = Some(port);
        }
        if let Some(user) = patch.user {
            self.user = Some(user);
        }
        if let Some(password) = patch.password {
            self.password = Some(password);
        }
        if let Some(dbname) = patch.dbname {
            self.dbname = Some(dbname);
        }
        if let Some(params) = patch.params {
            self.params.get_or_insert_with(BTreeMap::new).extend(params);
        }
        if let Some(file) = patch.file {
            self.file = Some(file);
        }
        if let Some(path) = patch.path {
            self.path = Some(path);
        }
        if let Some(server) = patch.server {
            self.server = Some(server);
        }

        if let Some(pool_patch) = patch.pool.filter(PoolCfg::has_any_value) {
            self.pool
                .get_or_insert_with(PoolCfg::default)
                .apply_patch(&pool_patch);
        }
    }
}

impl PoolCfg {
    #[must_use]
    pub const fn has_any_value(&self) -> bool {
        self.max_conns.is_some()
            || self.min_conns.is_some()
            || self.acquire_timeout.is_some()
            || self.idle_timeout.is_some()
            || self.max_lifetime.is_some()
            || self.test_before_acquire.is_some()
    }

    pub const fn apply_patch(&mut self, patch: &Self) {
        if let Some(max_conns) = patch.max_conns {
            self.max_conns = Some(max_conns);
        }
        if let Some(min_conns) = patch.min_conns {
            self.min_conns = Some(min_conns);
        }
        if let Some(acquire_timeout) = patch.acquire_timeout {
            self.acquire_timeout = Some(acquire_timeout);
        }
        if let Some(idle_timeout) = patch.idle_timeout {
            self.idle_timeout = Some(idle_timeout);
        }
        if let Some(max_lifetime) = patch.max_lifetime {
            self.max_lifetime = Some(max_lifetime);
        }
        if let Some(test_before_acquire) = patch.test_before_acquire {
            self.test_before_acquire = Some(test_before_acquire);
        }
    }
}

/// Per-vendor configuration bag: vendor name → arbitrary JSON/YAML value.
///
/// Each vendor's section can be deserialized into a typed struct via
/// [`AppConfig::vendor_config`] or [`AppConfig::vendor_config_or_default`].
pub type VendorConfig = HashMap<String, Value>;

/// Top-level `OpenTelemetry` configuration grouping resource identity,
/// a shared default exporter, tracing settings and metrics settings.
#[derive(Clone, Deserialize, Serialize, Default, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenTelemetryConfig {
    #[serde(default)]
    pub resource: OpenTelemetryResource,
    /// Default exporter shared by tracing and metrics. Per-signal `exporter`
    /// fields override this when present.
    pub exporter: Option<Exporter>,
    #[serde(default)]
    pub tracing: TracingConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}

/// `OpenTelemetry` resource identity — attached to all traces and metrics
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenTelemetryResource {
    /// Logical service name.
    #[serde(default = "default_service_name")]
    pub service_name: String,
    /// Extra resource attributes added to every span and metric data point.
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

/// Return the default OpenTelemetry service name used when none is configured
fn default_service_name() -> String {
    "gears".to_owned()
}

impl Default for OpenTelemetryResource {
    fn default() -> Self {
        Self {
            service_name: default_service_name(),
            attributes: BTreeMap::default(),
        }
    }
}

/// Tracing configuration for `OpenTelemetry` distributed tracing
#[derive(Clone, Deserialize, Serialize, Default, schemars::JsonSchema)]
pub struct TracingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exporter: Option<Exporter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampler: Option<Sampler>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub propagation: Option<Propagation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpOpts>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logs_correlation: Option<LogsCorrelation>,
}

/// Metrics configuration for `OpenTelemetry` metrics collection
#[derive(Clone, Deserialize, Serialize, Default, schemars::JsonSchema)]
pub struct MetricsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub exporter: Exporter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cardinality_limit: Option<usize>,
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExporterKind {
    #[default]
    OtlpGrpc,
    OtlpHttp,
}

/// Telemetry exporter (OTLP gRPC or HTTP).
#[derive(Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub struct Exporter {
    #[serde(default)]
    pub kind: ExporterKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Sampler {
    ParentBasedAlwaysOn {},
    ParentBasedRatio {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ratio: Option<f64>,
    },
    AlwaysOn {},
    AlwaysOff {},
}

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct Propagation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub w3c_trace_context: Option<bool>,
}

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct HttpOpts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inject_request_id_header: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_headers: Option<Vec<String>>,
}

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct LogsCorrelation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inject_trace_ids_into_logs: Option<bool>,
}
