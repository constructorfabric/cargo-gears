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
    Src(crate::source::SourceArgs),
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
            HelpCommand::Src(args) => args.run(),
            HelpCommand::Topic(args) => args.run(),
        }
    }
}

impl SchemaArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let text = match self.target {
            SchemaTarget::Manifest => schema_manifest(self.section.as_deref()),
            SchemaTarget::Config => schema_config(self.section.as_deref()),
            SchemaTarget::Module => schema_module(self.section.as_deref()),
        }?;
        println!("{text}");
        Ok(())
    }
}

impl TopicArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let text = match self.topic {
            Topic::Manifest => include_str!("topics/manifest.md"),
            Topic::ModuleRefs => include_str!("topics/module_refs.md"),
            Topic::GeneratedServer => include_str!("topics/generated_server.md"),
            Topic::Fips => include_str!("topics/fips.md"),
            Topic::Otel => include_str!("topics/otel.md"),
        };
        println!("{text}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Schema helpers — delegate to HelpSchema implementations
// ---------------------------------------------------------------------------

fn schema_manifest(section: Option<&str>) -> anyhow::Result<String> {
    use crate::manifest::{
        BuildPolicy, Environment, LintPolicy, Manifest, RunPolicy, TemplateRegistry, TestPolicy,
        Workspace,
    };
    match section {
        None => Ok(Manifest::help_text()),
        Some("workspace") => Ok(Workspace::help_text()),
        Some("apps") => {
            let mut out = Environment::help_text();
            out.push('\n');
            out.push_str(&RunPolicy::help_text());
            out.push('\n');
            out.push_str(&BuildPolicy::help_text());
            out.push('\n');
            out.push_str(&LintPolicy::help_text());
            out.push('\n');
            out.push_str(&TestPolicy::help_text());
            Ok(out)
        }
        Some("templates") => Ok(TemplateRegistry::help_text()),
        Some(other) => {
            bail!("unknown manifest section '{other}'; available: workspace, apps, templates")
        }
    }
}

fn schema_config(section: Option<&str>) -> anyhow::Result<String> {
    use crate::app_config::{
        AppConfig, DbConnConfig, Exporter, GlobalDatabaseConfig, MetricsConfig, ModuleConfig,
        ModuleRuntime, OpenTelemetryConfig, PoolCfg, ServerConfig, TracingConfig,
    };
    match section {
        None => Ok(AppConfig::help_text()),
        Some("server") => Ok(ServerConfig::help_text()),
        Some("database") => {
            let mut out = GlobalDatabaseConfig::help_text();
            out.push('\n');
            out.push_str(&DbConnConfig::help_text());
            out.push('\n');
            out.push_str(&PoolCfg::help_text());
            Ok(out)
        }
        Some("logging") => Ok("logging — Config Logging Section\n\n\
             Map of subsystem name → logging settings (JSON/YAML value).\n\
             The logging section is a free-form map; see config YAML examples."
            .to_owned()),
        Some("opentelemetry") => {
            let mut out = OpenTelemetryConfig::help_text();
            out.push('\n');
            out.push_str(&Exporter::help_text());
            out.push('\n');
            out.push_str(&TracingConfig::help_text());
            out.push('\n');
            out.push_str(&MetricsConfig::help_text());
            Ok(out)
        }
        Some("modules") => {
            let mut out = ModuleConfig::help_text();
            out.push('\n');
            out.push_str(&ModuleRuntime::help_text());
            Ok(out)
        }
        Some(other) => bail!(
            "unknown config section '{other}'; available: server, database, logging, opentelemetry, modules"
        ),
    }
}

fn schema_module(section: Option<&str>) -> anyhow::Result<String> {
    match section {
        None => Ok(include_str!("topics/module_schema.md").to_owned()),
        Some(other) => bail!("unknown module section '{other}'; no subsections available"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_manifest_returns_overview() {
        let text = schema_manifest(None).expect("overview should succeed");
        assert!(text.contains("Manifest"));
        assert!(text.contains("workspace"));
        assert!(text.contains("apps"));
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
        assert!(text.contains("AppConfig"));
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
        let topics: &[&str] = &[
            include_str!("topics/manifest.md"),
            include_str!("topics/module_refs.md"),
            include_str!("topics/generated_server.md"),
            include_str!("topics/fips.md"),
            include_str!("topics/otel.md"),
        ];
        for topic in topics {
            assert!(!topic.is_empty());
        }
    }

    #[test]
    fn help_schema_manifest_has_fields() {
        use crate::manifest::Manifest;
        let fields = Manifest::help_fields();
        assert!(!fields.is_empty());
        assert!(fields.iter().any(|f| f.name == "workspace"));
        assert!(fields.iter().any(|f| f.name == "apps"));
    }

    #[test]
    fn help_schema_appconfig_has_fields() {
        use crate::app_config::AppConfig;
        let fields = AppConfig::help_fields();
        assert!(!fields.is_empty());
        assert!(fields.iter().any(|f| f.name == "server"));
    }

    #[test]
    fn help_text_includes_doc_comments() {
        use crate::manifest::Workspace;
        let text = Workspace::help_text();
        assert!(text.contains("Workspace"));
        assert!(text.contains("Fields:"));
        assert!(text.contains("config-dir"));
    }
}
