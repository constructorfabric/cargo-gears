use anyhow::{Context, bail};
use schemars::{JsonSchema, schema_for};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpParams {
    pub command: HelpCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpCommand {
    Schema(SchemaParams),
    Src(crate::source::SourceParams),
    Topic(TopicParams),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaParams {
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
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct TopicParams {
    pub topic: Topic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Topic {
    Architecture,
    Cli,
    #[cfg_attr(feature = "clap", value(alias = "clienthub"))]
    ClientHub,
    Database,
    RestErrors,
    Fips,
    GearLayout,
    GearRefs,
    GearsCatalog,
    GeneratedServer,
    Lifecycle,
    Manifest,
    Otel,
    RestApi,
    Security,
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Architecture => f.write_str("architecture"),
            Self::Cli => f.write_str("cli"),
            Self::ClientHub => f.write_str("clienthub"),
            Self::Database => f.write_str("database"),
            Self::RestErrors => f.write_str("rest-errors"),
            Self::Fips => f.write_str("fips"),
            Self::GearLayout => f.write_str("gear-layout"),
            Self::GearRefs => f.write_str("gear-refs"),
            Self::GearsCatalog => f.write_str("gears-catalog"),
            Self::GeneratedServer => f.write_str("generated-server"),
            Self::Lifecycle => f.write_str("lifecycle"),
            Self::Manifest => f.write_str("manifest"),
            Self::Otel => f.write_str("otel"),
            Self::RestApi => f.write_str("rest-api"),
            Self::Security => f.write_str("security"),
        }
    }
}

impl HelpParams {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            HelpCommand::Schema(args) => args.run(),
            HelpCommand::Src(args) => args.run(),
            HelpCommand::Topic(args) => args.run(),
        }
    }
}

impl SchemaParams {
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

impl TopicParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let text = match self.topic {
            Topic::Architecture => include_str!("topics/architecture.md"),
            Topic::Cli => include_str!("topics/cli.md"),
            Topic::ClientHub => include_str!("topics/clienthub.md"),
            Topic::Database => include_str!("topics/database.md"),
            Topic::RestErrors => include_str!("topics/errors.md"),
            Topic::Fips => include_str!("topics/fips.md"),
            Topic::GearLayout => include_str!("topics/gear_layout.md"),
            Topic::GearRefs => include_str!("topics/gear_refs.md"),
            Topic::GearsCatalog => include_str!("topics/gears_catalog.md"),
            Topic::GeneratedServer => include_str!("topics/generated_server.md"),
            Topic::Lifecycle => include_str!("topics/lifecycle.md"),
            Topic::Manifest => include_str!("topics/manifest.md"),
            Topic::Otel => include_str!("topics/otel.md"),
            Topic::RestApi => include_str!("topics/rest_api.md"),
            Topic::Security => include_str!("topics/security.md"),
        };
        println!("{text}");
        Ok(())
    }
}

fn schema_manifest(section: Option<&str>) -> anyhow::Result<String> {
    use crate::manifest::{Environment, Manifest, TemplateRegistry, Workspace};
    match section {
        None => to_json_schema::<Manifest>(),
        Some("workspace") => to_json_schema::<Workspace>(),
        Some("apps") => to_json_schema::<Environment>(),
        Some("templates") => to_json_schema::<TemplateRegistry>(),
        Some(other) => {
            bail!("unknown manifest section '{other}'; available: workspace, apps, templates")
        }
    }
}

fn to_json_schema<T: JsonSchema>() -> anyhow::Result<String> {
    let sch = schema_for!(T);
    serde_json::to_string(&sch).context("failed to serialize schema")
}

fn schema_config(section: Option<&str>) -> anyhow::Result<String> {
    use crate::app_config::{
        AppConfig, GearConfig, GlobalDatabaseConfig, LoggingConfig, OpenTelemetryConfig,
        ServerConfig,
    };
    match section {
        None => to_json_schema::<AppConfig>(),
        Some("server") => to_json_schema::<ServerConfig>(),
        Some("database") => to_json_schema::<GlobalDatabaseConfig>(),
        Some("logging") => to_json_schema::<LoggingConfig>(),
        Some("opentelemetry") => to_json_schema::<OpenTelemetryConfig>(),
        Some("gears") => to_json_schema::<GearConfig>(),
        Some(other) => bail!(
            "unknown config section '{other}'; available: server, database, logging, opentelemetry, gears"
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
        assert!(text.contains("gears"));
    }

    #[test]
    fn schema_config_sections_resolve() {
        for section in &["server", "database", "logging", "opentelemetry", "gears"] {
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
            include_str!("topics/architecture.md"),
            include_str!("topics/cli.md"),
            include_str!("topics/clienthub.md"),
            include_str!("topics/database.md"),
            include_str!("topics/errors.md"),
            include_str!("topics/fips.md"),
            include_str!("topics/gear_layout.md"),
            include_str!("topics/gear_refs.md"),
            include_str!("topics/gears_catalog.md"),
            include_str!("topics/generated_server.md"),
            include_str!("topics/lifecycle.md"),
            include_str!("topics/manifest.md"),
            include_str!("topics/otel.md"),
            include_str!("topics/rest_api.md"),
            include_str!("topics/security.md"),
        ];
        for topic in topics {
            assert!(!topic.is_empty());
        }
    }
}
