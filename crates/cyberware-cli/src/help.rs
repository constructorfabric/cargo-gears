use crate::common::Registry;
use clap::{Args, Subcommand, ValueEnum};
use semver::Version;
use std::path::PathBuf;

#[derive(Args)]
pub struct HelpArgs {
    #[command(subcommand)]
    command: HelpCommand,
}

impl HelpArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::CyberfabricCommand::from(self).run()
    }
}

impl From<HelpArgs> for cyberware_cli_core::CyberfabricCommand {
    fn from(args: HelpArgs) -> Self {
        Self::Help(cyberware_cli_core::help::HelpArgs {
            command: args.command.into(),
        })
    }
}

#[derive(Subcommand)]
pub enum HelpCommand {
    /// Print the schema for manifest, config, or module formats
    Schema(SchemaArgs),
    /// Resolve Rust source code from a crate (alias for top-level `docs`)
    Docs(DocsArgs),
    /// Print operational documentation for a topic
    Topic(TopicArgs),
}

impl From<HelpCommand> for cyberware_cli_core::help::HelpCommand {
    fn from(command: HelpCommand) -> Self {
        match command {
            HelpCommand::Schema(args) => Self::Schema(args.into()),
            HelpCommand::Docs(args) => Self::Docs(args.into()),
            HelpCommand::Topic(args) => Self::Topic(args.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct SchemaArgs {
    /// Schema target to display
    target: SchemaTarget,
    /// Drill into a specific section of the schema
    #[arg(long)]
    section: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum SchemaTarget {
    Manifest,
    Config,
    Module,
}

impl From<SchemaArgs> for cyberware_cli_core::help::SchemaArgs {
    fn from(args: SchemaArgs) -> Self {
        Self {
            target: args.target.into(),
            section: args.section,
        }
    }
}

impl From<SchemaTarget> for cyberware_cli_core::help::SchemaTarget {
    fn from(target: SchemaTarget) -> Self {
        match target {
            SchemaTarget::Manifest => Self::Manifest,
            SchemaTarget::Config => Self::Config,
            SchemaTarget::Module => Self::Module,
        }
    }
}

// ---------------------------------------------------------------------------
// Docs (alias for top-level docs command)
// ---------------------------------------------------------------------------

#[derive(Args)]
#[command(disable_version_flag = true)]
pub struct DocsArgs {
    /// Path to the Cargo workspace or crate to inspect
    #[arg(short = 'p', long, default_value = ".")]
    path: PathBuf,
    /// Registry to query when the crate is not present in local metadata
    #[arg(long, value_enum, default_value_t = Registry::CratesIo)]
    registry: Registry,
    /// Print query/package/version/source metadata before the resolved Rust source
    #[arg(short = 'v', long)]
    verbose: bool,
    /// List `library_name` -> `package_name` mappings for a package query
    #[arg(short = 'l', long)]
    libs: bool,
    /// Resolve a specific crate version after metadata/cache lookup misses
    #[arg(long)]
    version: Option<Version>,
    /// Remove the docs cache for the selected registry before resolving
    #[arg(long)]
    clean: bool,
    /// Rust path to resolve (start with the package name)
    query: Option<String>,
}

impl From<DocsArgs> for cyberware_cli_core::docs::DocsArgs {
    fn from(args: DocsArgs) -> Self {
        Self {
            path: args.path,
            registry: args.registry,
            verbose: args.verbose,
            libs: args.libs,
            version: args.version,
            clean: args.clean,
            query: args.query,
        }
    }
}

// ---------------------------------------------------------------------------
// Topic
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct TopicArgs {
    /// Topic to display
    topic: Topic,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Topic {
    Manifest,
    #[value(name = "module-refs")]
    ModuleRefs,
    #[value(name = "generated-server")]
    GeneratedServer,
    Fips,
    Otel,
}

impl From<TopicArgs> for cyberware_cli_core::help::TopicArgs {
    fn from(args: TopicArgs) -> Self {
        Self {
            topic: args.topic.into(),
        }
    }
}

impl From<Topic> for cyberware_cli_core::help::Topic {
    fn from(topic: Topic) -> Self {
        match topic {
            Topic::Manifest => Self::Manifest,
            Topic::ModuleRefs => Self::ModuleRefs,
            Topic::GeneratedServer => Self::GeneratedServer,
            Topic::Fips => Self::Fips,
            Topic::Otel => Self::Otel,
        }
    }
}
