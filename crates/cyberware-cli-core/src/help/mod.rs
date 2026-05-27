use anyhow::bail;
use std::fmt;
use std::fmt::Write;

// ---------------------------------------------------------------------------
// HelpSchema trait — implemented by the derive macro
// ---------------------------------------------------------------------------

/// Metadata for a single struct field or enum variant, produced by the derive
/// macro from doc comments and serde attributes.
#[derive(Debug, Clone)]
pub struct FieldHelp {
    pub name: &'static str,
    pub field_type: &'static str,
    pub doc: &'static str,
    pub optional: bool,
    pub has_default: bool,
}

/// Trait implemented by `#[derive(HelpSchema)]` on schema types.
/// Provides structured documentation harvested from doc comments and serde
/// attributes at compile time.
pub trait HelpSchema {
    /// The Rust type name (e.g. `"Manifest"`).
    fn help_name() -> &'static str;
    /// Concatenated struct/enum-level doc comments.
    fn help_doc() -> &'static str;
    /// Per-field (or per-variant) metadata.
    fn help_fields() -> Vec<FieldHelp>;

    /// Render a human-readable help text from the harvested metadata.
    #[must_use]
    fn help_text() -> String {
        let mut out = String::new();
        out.push_str(Self::help_name());
        out.push('\n');
        let doc = Self::help_doc();
        if !doc.is_empty() {
            out.push('\n');
            out.push_str(doc);
            out.push('\n');
        }
        let fields = Self::help_fields();
        if !fields.is_empty() {
            out.push_str("\nFields:\n");
            for f in &fields {
                let qualifier = if f.optional {
                    "optional"
                } else if f.has_default {
                    "default"
                } else {
                    "required"
                };
                let _ = writeln!(
                    out,
                    "  {:<24} {:<28} {}{}",
                    f.name,
                    f.field_type,
                    qualifier,
                    if f.doc.is_empty() {
                        String::new()
                    } else {
                        format!("  — {}", f.doc)
                    },
                );
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Command types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpArgs {
    pub command: HelpCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpCommand {
    Schema(SchemaArgs),
    Docs(crate::docs::DocsArgs),
    Topic(TopicArgs),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaArgs {
    pub target: SchemaTarget,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaTarget {
    Manifest,
    Config,
    Module,
}

impl fmt::Display for SchemaTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest => f.write_str("manifest"),
            Self::Config => f.write_str("config"),
            Self::Module => f.write_str("module"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicArgs {
    pub topic: Topic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Manifest,
    ModuleRefs,
    GeneratedServer,
    Fips,
    Otel,
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest => f.write_str("manifest"),
            Self::ModuleRefs => f.write_str("module-refs"),
            Self::GeneratedServer => f.write_str("generated-server"),
            Self::Fips => f.write_str("fips"),
            Self::Otel => f.write_str("otel"),
        }
    }
}

impl HelpArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            HelpCommand::Schema(args) => args.run(),
            HelpCommand::Docs(args) => args.run(),
            HelpCommand::Topic(args) => args.run(),
        }
    }
}

impl SchemaArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let section = self.section.as_deref();
        let text = match self.target {
            SchemaTarget::Manifest => schema_manifest(section),
            SchemaTarget::Config => schema_config(section),
            SchemaTarget::Module => schema_module(section),
        };
        match text {
            Ok(text) => {
                println!("{text}");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl TopicArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let text = match self.topic {
            Topic::Manifest => TOPIC_MANIFEST,
            Topic::ModuleRefs => TOPIC_MODULE_REFS,
            Topic::GeneratedServer => TOPIC_GENERATED_SERVER,
            Topic::Fips => TOPIC_FIPS,
            Topic::Otel => TOPIC_OTEL,
        };
        println!("{text}");
        Ok(())
    }
}

fn schema_manifest(section: Option<&str>) -> anyhow::Result<&'static str> {
    match section {
        None => Ok(SCHEMA_MANIFEST),
        Some("workspace") => Ok(SCHEMA_MANIFEST_WORKSPACE),
        Some("apps") => Ok(SCHEMA_MANIFEST_APPS),
        Some("templates") => Ok(SCHEMA_MANIFEST_TEMPLATES),
        Some(other) => {
            bail!("unknown manifest section '{other}'; available: workspace, apps, templates")
        }
    }
}

fn schema_config(section: Option<&str>) -> anyhow::Result<&'static str> {
    match section {
        None => Ok(SCHEMA_CONFIG),
        Some("server") => Ok(SCHEMA_CONFIG_SERVER),
        Some("database") => Ok(SCHEMA_CONFIG_DATABASE),
        Some("logging") => Ok(SCHEMA_CONFIG_LOGGING),
        Some("opentelemetry") => Ok(SCHEMA_CONFIG_OTEL),
        Some("modules") => Ok(SCHEMA_CONFIG_MODULES),
        Some(other) => bail!(
            "unknown config section '{other}'; available: server, database, logging, opentelemetry, modules"
        ),
    }
}

fn schema_module(section: Option<&str>) -> anyhow::Result<&'static str> {
    match section {
        None => Ok(SCHEMA_MODULE),
        Some(other) => bail!("unknown module section '{other}'; no subsections available"),
    }
}

// ---------------------------------------------------------------------------
// Schema: manifest (Cyberware.toml)
// ---------------------------------------------------------------------------

const SCHEMA_MANIFEST: &str = "\
Cyberware.toml — Manifest Schema (v1)

The manifest file drives build, run, lint, and test workflows. It declares
workspace-level defaults and per-app/environment overrides.

Top-level sections:
  [workspace]   Workspace paths and defaults
  [apps]        App/environment matrix
  [templates]   Optional template registry

Use --section <name> to inspect a specific section.

Example:
  [workspace]
  generated-dir = \".cyberware\"

  [apps.myapp.dev]
  config = \"app-dev.yml\"
  modules = [
    { source = \"local\", name = \"rest-gateway\" },
  ]";

const SCHEMA_MANIFEST_WORKSPACE: &str = "\
[workspace] — Manifest Workspace Section

Fields:
  version        u32, default 1         Schema version
  root           path, optional         Workspace root override (relative to manifest dir)
  config-dir     path, default \"config\" Directory containing config YAML files
  generated-dir  path, default \".cyberware\"
                                        Directory for generated server projects

Example:
  [workspace]
  version = 1
  config-dir = \"config\"
  generated-dir = \".cyberware\"";

const SCHEMA_MANIFEST_APPS: &str = "\
[apps] — Manifest Apps Section

Structure: apps.<app_name>.<env_name> = Environment

Environment fields:
  config         path (required)        Config YAML path relative to config-dir
  modules        array of ModuleRef     Modules to include in the generated server
  run            RunPolicy, optional    Runtime policy overrides
  build          BuildPolicy, optional  Build policy overrides
  lint           LintPolicy             Lint policy (defaults enabled)
  test           TestPolicy             Test policy

ModuleRef (tagged by \"source\"):
  Local:  { source = \"local\",  name = \"...\", version = \"...\", package = \"...\" }
  Remote: { source = \"remote\", name = \"...\", version = \"...\", package = \"...\" }

RunPolicy:
  watch.enabled  bool, default true     Enable file watching in run mode
  watch.paths    array of paths         Extra paths to watch
  watch.ignore   array of paths         Paths to ignore
  fips           bool, default false    Enable FIPS mode
  otel           bool, default false    Enable OpenTelemetry

BuildPolicy:
  name           string, optional       Override generated project name (default: <app>-<env>)
  profile        \"debug\"|\"release\"|custom  Build profile
  clean          bool, optional         Remove Cargo.lock before build

LintPolicy:
  clippy         bool, default true     Run clippy
  fmt            bool, default true     Run cargo fmt --check
  feature-set-test  bool, default true  Run feature-set tests
  dylint         { enabled, skip }      Dylint rule configuration

TestPolicy:
  runner         \"cargo\"|\"nextest\"      Test runner
  coverage       bool, default false    Enable coverage
  feature-set    map of feature sets    Per-module feature combinations
  custom-command string, optional       Custom test command

Example:
  [apps.myapp.dev]
  config = \"app-dev.yml\"
  modules = [
    { source = \"local\", name = \"rest-gateway\" },
    { source = \"remote\", name = \"credstore\", package = \"cf-credstore\", version = \"0.4\" },
  ]

  [apps.myapp.dev.run]
  otel = true

  [apps.myapp.dev.build]
  profile = \"release\"
  clean = true";

const SCHEMA_MANIFEST_TEMPLATES: &str = "\
[templates] — Manifest Templates Section

Optional registry of template sources for generate commands.

Structure:
  [templates.module.<name>]   Module template sources
  [templates.config.<name>]   Config template sources
  [templates.agents.<name>]   Agent template sources

TemplateSource (tagged by \"source\"):
  Git:      { source = \"git\", url, revision?, tag?, branch?, subfolder? }
  Local:    { source = \"local\", path }
  Embedded: { source = \"embedded\" }

Example:
  [templates.module.custom-worker]
  source = \"git\"
  url = \"https://github.com/org/templates.git\"
  branch = \"main\"
  subfolder = \"Modules/custom-worker\"";

// ---------------------------------------------------------------------------
// Schema: config (YAML)
// ---------------------------------------------------------------------------

const SCHEMA_CONFIG: &str = "\
Runtime Configuration — YAML Schema

The config YAML file is the runtime configuration for the generated server.
It is referenced by the manifest's config field and passed to the server via
the CF_CLI_CONFIG environment variable.

Top-level sections:
  server          Core server settings
  database        Global database servers (optional)
  logging         Logging subsystem configuration
  opentelemetry   OpenTelemetry tracing and metrics (optional)
  modules_dir     Directory for per-module YAML overrides (optional)
  modules         Per-module configuration
  vendor          Vendor-specific configuration bags

Use --section <name> to inspect a specific section.

Example:
  server:
    home_dir: .cyberfabric
  logging:
    default:
      console_level: info
  modules: {}";

const SCHEMA_CONFIG_SERVER: &str = "\
server — Config Server Section

Fields:
  home_dir    path, default \".cyberfabric\"   Server home directory

Example:
  server:
    home_dir: .cyberfabric";

const SCHEMA_CONFIG_DATABASE: &str = "\
database — Config Database Section

Global database server configuration.

Fields:
  servers          map of DbConnConfig     Named database connections
  auto_provision   bool, optional          Auto-provision DB/schema in dev

DbConnConfig fields:
  engine           postgres|mysql|sqlite   Database engine
  dsn              string, optional        Full DSN connection string
  host             string, optional        Database host
  port             u16, optional           Database port
  user             string, optional        Database user
  password         string, optional        Password (supports ${VAR} expansion)
  dbname           string, optional        Database name
  params           map, optional           Extra connection parameters
  file             string, optional        SQLite file name
  path             path, optional          SQLite file path
  pool             PoolCfg, optional       Connection pool settings
  server           string, optional        Reference to a named global server

PoolCfg fields:
  max_conns              u32, optional
  min_conns              u32, optional
  acquire_timeout        duration, optional
  idle_timeout           duration, optional
  max_lifetime           duration, optional
  test_before_acquire    bool, optional

Example:
  database:
    servers:
      main:
        engine: postgres
        host: localhost
        port: 5432
        user: postgres
        password: ${DB_PASSWORD}
        dbname: app
        pool:
          max_conns: 20";

const SCHEMA_CONFIG_LOGGING: &str = "\
logging — Config Logging Section

Map of subsystem name to logging settings (JSON/YAML value).

Default subsystem: \"default\"

Common fields per subsystem:
  console_level    string     Console log level (trace, debug, info, warn, error)
  file             string     Log file path
  file_level       string     File log level
  max_age_days     integer    Maximum age of log files in days
  max_backups      integer    Maximum number of backup log files
  max_size_mb      integer    Maximum size of a log file in megabytes

Example:
  logging:
    default:
      console_level: info
      file: logs/cyberfabric.log
      file_level: debug
      max_age_days: 7
      max_backups: 3
      max_size_mb: 100";

const SCHEMA_CONFIG_OTEL: &str = "\
opentelemetry — Config OpenTelemetry Section

Fields:
  resource.service_name    string, default \"cyberfabric\"
  resource.attributes      map of string -> string
  exporter                 Exporter, optional (shared default)
  tracing                  TracingConfig
  metrics                  MetricsConfig

Exporter:
  kind        otlp_grpc|otlp_http, default otlp_grpc
  endpoint    string, optional
  headers     map of string -> string
  timeout_ms  u64, optional

TracingConfig:
  enabled        bool, default false
  exporter       Exporter, optional (overrides shared)
  sampler        parent_based_always_on|parent_based_ratio|always_on|always_off
  propagation    { w3c_trace_context: bool }
  http           { inject_request_id_header, record_headers }
  logs_correlation  { inject_trace_ids_into_logs: bool }

MetricsConfig:
  enabled            bool, default false
  exporter           Exporter
  cardinality_limit  usize, optional

Example:
  opentelemetry:
    resource:
      service_name: cyberfabric
    tracing:
      enabled: true
      sampler:
        parent_based_ratio:
          ratio: 0.1
    metrics:
      enabled: true";

const SCHEMA_CONFIG_MODULES: &str = "\
modules — Config Modules Section

Per-module configuration bag: module_name -> ModuleConfig

ModuleConfig fields:
  database    DbConnConfig, optional     Module-level database connection
  config      value, default {}          Free-form module configuration
  runtime     ModuleRuntime, optional    Module runtime type
  metadata    ConfigModuleMetadata       Cargo metadata for server generation

ModuleRuntime:
  type         local|oop, default local  Runtime kind
  execution    ExecutionConfig, optional For out-of-process modules

ExecutionConfig:
  executable_path     string             Path to the executable
  args                array of string    Command-line arguments
  working_directory   string, optional   Working directory
  environment         map                Environment variables

ConfigModuleMetadata:
  package             string             Cargo package name
  version             string             Crate version
  features            array of string    Cargo features to enable
  default_features    bool, optional     Use default features
  path                string, optional   Local crate path

Example:
  modules:
    rest-gateway:
      config:
        port: 8080
      metadata:
        package: cf-rest-gateway
        version: \"0.4.0\"";

// ---------------------------------------------------------------------------
// Schema: module
// ---------------------------------------------------------------------------

const SCHEMA_MODULE: &str = "\
Module Schema

A CyberFabric module is a Rust crate inside the workspace's modules/ directory.
Module metadata is discovered from Cargo.toml and workspace metadata.

Module Cargo.toml structure:
  [package]
  name         string    Crate name (used as the module identifier)
  version      string    Crate version

  [dependencies]
  ...                    Dependencies are promoted to workspace.dependencies
                         during `generate module`

  [package.metadata.cyberware]
  deps           array of string        Module dependencies (other module names)
  capabilities   array of string        Declared capabilities (e.g. \"grpc\", \"http\")

Discovery:
  The CLI discovers modules by running `cargo metadata` on the workspace and
  finding packages whose manifest path is under the workspace root.

  Module names in config and manifest reference the Cargo package name.

Example Cargo.toml:
  [package]
  name = \"rest-gateway\"
  version = \"0.1.0\"
  edition = \"2024\"

  [dependencies]
  cf-modkit = { workspace = true, features = [\"http\"] }

  [package.metadata.cyberware]
  deps = [\"authn-resolver\"]
  capabilities = [\"http\"]";

// ---------------------------------------------------------------------------
// Topics
// ---------------------------------------------------------------------------

const TOPIC_MANIFEST: &str = "\
Topic: Manifest (Cyberware.toml)

The Cyberware.toml manifest is the central configuration for multi-environment
builds. It replaces the need to pass --config to build/run by declaring all
generation inputs in one file.

Key concepts:
  - One manifest per workspace, usually at the workspace root
  - Apps group environments: apps.<app>.<env>
  - Each environment declares its config path, modules, and policies
  - The manifest controls the generated-dir, build profile, watch mode, etc.

Workflow:
  1. Write Cyberware.toml with your apps and environments
  2. Run: cargo cyberfabric build --app myapp --env dev
  3. The CLI resolves the manifest, discovers modules, and generates the server
  4. Use --dry-run to inspect the generated structure without building

Commands that use the manifest:
  cargo cyberfabric build --app <APP> --env <ENV>
  cargo cyberfabric run   --app <APP> --env <ENV>
  cargo cyberfabric manifest validate
  cargo cyberfabric manifest ls

See also: cargo cyberfabric help schema manifest";

const TOPIC_MODULE_REFS: &str = "\
Topic: Module References

Modules are referenced in the manifest and config in two forms:

Local modules:
  Discovered from the workspace via `cargo metadata`. The CLI scans for
  packages whose manifest path is under the workspace root.

  Manifest syntax:
    { source = \"local\", name = \"rest-gateway\" }

  The name must match a Cargo package name discoverable in the workspace.
  Optional overrides: version, package.

Remote modules:
  Downloaded from a registry (currently only crates.io). Not present in the
  workspace.

  Manifest syntax:
    { source = \"remote\", name = \"credstore\", package = \"cf-credstore\", version = \"0.4\" }

  Required fields: name, package, version.

Config module metadata:
  When using config-driven builds (deploy), modules need metadata in the YAML:
    modules:
      rest-gateway:
        metadata:
          package: cf-rest-gateway
          version: \"0.4.0\"

  With manifest-driven builds (build/run), metadata is resolved automatically
  from the workspace or the remote module reference.";

const TOPIC_GENERATED_SERVER: &str = "\
Topic: Generated Server

The CLI generates an ephemeral Cargo project that aggregates your modules into
a single runnable binary.

Location:
  <workspace>/<generated-dir>/<name>/
  Default: .cyberware/<app>-<env>/

Generated files:
  Cargo.toml           Declares dependencies on all selected modules
  .cargo/config.toml   Points target-dir back to workspace target/
  src/main.rs          Bootstraps the modkit server with all modules

The generated server reads its runtime config from the CF_CLI_CONFIG
environment variable, which the CLI sets automatically during build/run.

Key points:
  - The generated project is ephemeral; regenerated on every build/run
  - Dependencies are rewritten to workspace-relative paths
  - The --name flag overrides the default <app>-<env> project name
  - Use --dry-run to inspect without building
  - The generated-dir is controlled by manifest workspace.generated-dir

Manual execution:
  If you run the compiled binary directly, you must set CF_CLI_CONFIG:
    CF_CLI_CONFIG=config/app-dev.yml ./target/debug/app-dev";

const TOPIC_FIPS: &str = "\
Topic: FIPS Mode

FIPS (Federal Information Processing Standards) mode enables FIPS-compliant
cryptography in the generated server.

How it works:
  - The CLI passes -F fips to cargo build/run, enabling the fips Cargo feature
  - The modkit framework's fips feature activates FIPS-compliant TLS and crypto

Activation:
  CLI flag:      --fips / --no-fips
  Manifest:      [apps.myapp.dev.run] fips = true
  Priority:      CLI flag > manifest policy > default (false)

Examples:
  cargo cyberfabric run --app myapp --env dev --fips
  cargo cyberfabric build --app myapp --env prod --fips";

const TOPIC_OTEL: &str = "\
Topic: OpenTelemetry (OTel)

OpenTelemetry support enables distributed tracing and metrics collection in
the generated server.

How it works:
  - The CLI passes -F otel to cargo build/run, enabling the otel Cargo feature
  - The modkit framework's otel feature activates tracing and metrics exporters
  - Runtime configuration is in the config YAML under the opentelemetry section

Activation:
  CLI flag:      --otel / --no-otel
  Manifest:      [apps.myapp.dev.run] otel = true
  Priority:      CLI flag > manifest policy > default (false)

Runtime config:
  opentelemetry:
    resource:
      service_name: my-service
    tracing:
      enabled: true
      sampler:
        parent_based_ratio:
          ratio: 0.1
    metrics:
      enabled: true

See also: cargo cyberfabric help schema config --section opentelemetry";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_manifest_returns_overview() {
        let text = schema_manifest(None).expect("overview should succeed");
        assert!(text.contains("Cyberware.toml"));
        assert!(text.contains("[workspace]"));
        assert!(text.contains("[apps]"));
    }

    #[test]
    fn schema_manifest_sections_resolve() {
        for section in &["workspace", "apps", "templates"] {
            assert!(
                schema_manifest(Some(section)).is_ok(),
                "section '{section}' should resolve"
            );
        }
    }

    #[test]
    fn schema_manifest_unknown_section_fails() {
        assert!(schema_manifest(Some("bogus")).is_err());
    }

    #[test]
    fn schema_config_returns_overview() {
        let text = schema_config(None).expect("overview should succeed");
        assert!(text.contains("Runtime Configuration"));
        assert!(text.contains("server"));
        assert!(text.contains("modules"));
    }

    #[test]
    fn schema_config_sections_resolve() {
        for section in &["server", "database", "logging", "opentelemetry", "modules"] {
            assert!(
                schema_config(Some(section)).is_ok(),
                "section '{section}' should resolve"
            );
        }
    }

    #[test]
    fn schema_config_unknown_section_fails() {
        assert!(schema_config(Some("bogus")).is_err());
    }

    #[test]
    fn schema_module_returns_overview() {
        let text = schema_module(None).expect("overview should succeed");
        assert!(text.contains("Module Schema"));
        assert!(text.contains("Cargo.toml"));
    }

    #[test]
    fn schema_module_unknown_section_fails() {
        assert!(schema_module(Some("bogus")).is_err());
    }

    #[test]
    fn all_topics_are_non_empty() {
        for topic in &[
            TOPIC_MANIFEST,
            TOPIC_MODULE_REFS,
            TOPIC_GENERATED_SERVER,
            TOPIC_FIPS,
            TOPIC_OTEL,
        ] {
            assert!(!topic.is_empty());
        }
    }
}
