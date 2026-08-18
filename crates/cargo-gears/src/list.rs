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
    /// List Cargo packages for a gear/lib scope
    Packages(PackagesArgs),
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
pub struct PackagesArgs {
    #[command(flatten)]
    workspace: WorkspacePath,
    /// Comma-separated directory paths to scope the search (relative to workspace root).
    /// Only packages whose manifest lives under one of these directories are included.
    /// When omitted, all workspace packages are listed.
    #[arg(long, value_delimiter = ',')]
    scope_dirs: Vec<String>,
    /// Skip transitive reverse dependencies; print only the matched packages
    #[arg(long)]
    no_rdeps: bool,
    /// Print as `-p <pkg>` flags suitable for cargo commands
    #[arg(long = "cargo-flags")]
    cargo_flags: bool,
}

impl From<ListCommand> for cargo_gears_core::list::ListCommand {
    fn from(command: ListCommand) -> Self {
        match command {
            ListCommand::Gears(args) => Self::Gears(args.into()),
            ListCommand::Templates(args) => Self::Templates(args.into()),
            ListCommand::Packages(args) => Self::Packages(args.into()),
        }
    }
}

impl From<PackagesArgs> for cargo_gears_core::list::PackagesParams {
    fn from(args: PackagesArgs) -> Self {
        Self {
            path: args.workspace.path,
            scope_dirs: args.scope_dirs,
            include_rdeps: !args.no_rdeps,
            cargo_flag_format: args.cargo_flags,
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
        }
    }
}
