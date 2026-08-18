use crate::common::{OutputFormat, Registry, WorkspacePath};
use clap::{ArgAction, Args, Subcommand};

#[derive(Args)]
pub struct ListArgs {
    #[command(subcommand)]
    command: ListCommand,
}

#[derive(Subcommand)]
pub enum ListCommand {
    /// List gears available
    Gears(GearsArgs),
    /// List templates available
    Templates(TemplatesArgs),
    /// List feature names from a Cargo.toml
    Features(FeaturesArgs),
    /// List dependencies from a Cargo.toml
    Deps(DepsArgs),
}

#[derive(Args)]
pub struct TemplatesArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
}

#[derive(Args)]
pub struct GearsArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    /// Show all information related to the gears (fetches registry metadata for system gears)
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Only list built-in system gears from the registry
    #[arg(long, action = ArgAction::SetTrue)]
    system: bool,
    /// Only list workspace-discovered gears
    #[arg(long, action = ArgAction::SetTrue)]
    local: bool,
    /// Registry to query for system-crate metadata
    #[arg(long, value_enum, default_value_t = Registry::CratesIo)]
    registry: Registry,
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
    /// Filter gear names by regex pattern
    #[arg(long)]
    filter: Option<String>,
    /// Comma-separated directory paths to scope the search (relative to workspace root)
    #[arg(long, value_delimiter = ',')]
    scope_dirs: Vec<String>,
    /// Include transitive reverse dependencies of matched gears
    #[arg(long)]
    include_rdeps: bool,
}

impl ListArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cargo_gears_core::list::ListParams::from(self).run()
    }
}

impl From<ListArgs> for cargo_gears_core::list::ListParams {
    fn from(args: ListArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

#[derive(Args)]
pub struct FeaturesArgs {
    /// Path to Cargo.toml to inspect
    #[arg(long)]
    manifest: std::path::PathBuf,
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::List)]
    format: OutputFormat,
}

#[derive(Args)]
pub struct DepsArgs {
    /// Path to Cargo.toml to inspect
    #[arg(long)]
    manifest: std::path::PathBuf,
    /// Only list non-optional (always-linked) dependencies
    #[arg(long)]
    non_optional: bool,
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::List)]
    format: OutputFormat,
}

impl From<ListCommand> for cargo_gears_core::list::ListCommand {
    fn from(command: ListCommand) -> Self {
        match command {
            ListCommand::Gears(args) => Self::Gears(args.into()),
            ListCommand::Templates(args) => Self::Templates(args.into()),
            ListCommand::Features(args) => Self::Features(args.into()),
            ListCommand::Deps(args) => Self::Deps(args.into()),
        }
    }
}

impl From<FeaturesArgs> for cargo_gears_core::list::FeaturesParams {
    fn from(args: FeaturesArgs) -> Self {
        Self {
            manifest: args.manifest,
            format: args.format,
        }
    }
}

impl From<DepsArgs> for cargo_gears_core::list::DepsParams {
    fn from(args: DepsArgs) -> Self {
        Self {
            manifest: args.manifest,
            non_optional: args.non_optional,
            format: args.format,
        }
    }
}

impl From<TemplatesArgs> for cargo_gears_core::list::TemplatesParams {
    fn from(args: TemplatesArgs) -> Self {
        Self {
            path: args.workspace.path,
        }
    }
}

impl From<GearsArgs> for cargo_gears_core::list::GearsParams {
    fn from(args: GearsArgs) -> Self {
        let output = if args.system || args.local {
            match (args.system, args.local) {
                (true, false) => cargo_gears_core::list::GearsOutput::system(),
                (false, true) => cargo_gears_core::list::GearsOutput::local(),
                _ => cargo_gears_core::list::GearsOutput::all(),
            }
        } else {
            cargo_gears_core::list::GearsOutput::all()
        };

        Self {
            path: args.workspace.path,
            verbose: args.verbose,
            output,
            registry: args.registry,
            format: args.format,
            filter: args.filter,
            scope_dirs: args.scope_dirs,
            include_rdeps: args.include_rdeps,
        }
    }
}
