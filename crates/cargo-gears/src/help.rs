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
        cargo_gears_core::GearsCommand::from(self).run()
    }
}

impl From<HelpArgs> for cargo_gears_core::GearsCommand {
    fn from(args: HelpArgs) -> Self {
        Self::Help(cargo_gears_core::help::HelpParams {
            command: args.command.into(),
        })
    }
}

#[derive(Subcommand)]
pub enum HelpCommand {
    /// Print the schema for manifest, config, or module formats
    Schema(SchemaArgs),
    /// Resolve Rust source code from a crate (alias for top-level `src`)
    Src(SrcArgs),
    /// Print operational documentation for a topic
    Topic(cargo_gears_core::help::TopicParams),
}

impl From<HelpCommand> for cargo_gears_core::help::HelpCommand {
    fn from(command: HelpCommand) -> Self {
        match command {
            HelpCommand::Schema(args) => Self::Schema(args.into()),
            HelpCommand::Src(args) => Self::Src(args.into()),
            HelpCommand::Topic(args) => Self::Topic(args),
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

impl From<SchemaArgs> for cargo_gears_core::help::SchemaParams {
    fn from(args: SchemaArgs) -> Self {
        Self {
            target: args.target.into(),
            section: args.section,
        }
    }
}

impl From<SchemaTarget> for cargo_gears_core::help::SchemaTarget {
    fn from(target: SchemaTarget) -> Self {
        match target {
            SchemaTarget::Manifest => Self::Manifest,
            SchemaTarget::Config => Self::Config,
            SchemaTarget::Module => Self::Module,
        }
    }
}

// ---------------------------------------------------------------------------
// Src (alias for top-level src command)
// ---------------------------------------------------------------------------

#[derive(Args)]
#[command(disable_version_flag = true)]
pub struct SrcArgs {
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
    /// Remove the source cache for the selected registry before resolving
    #[arg(long)]
    clean: bool,
    /// Rust path to resolve (start with the package name)
    query: Option<String>,
}

impl From<SrcArgs> for cargo_gears_core::source::SourceParams {
    fn from(args: SrcArgs) -> Self {
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
