use crate::common::Registry;
use clap::Args;
use semver::Version;
use std::path::PathBuf;

#[derive(Args)]
#[command(disable_version_flag = true)]
/// Resolve Rust source code from a crate
pub struct SourceArgs {
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
    /// Rust path to resolve(start always by `package_name`), for example `cf-modkit` it will resolve the lib.rs
    /// You can resolve modules `tokio::sync` to resolve the source code from the sync module from tokio crate
    /// You can also resolve by function name, for example `cf-modkit::gts::plugin::BaseModkitPluginV1`
    /// Also resolve by function name, for instance `cf-modkit::gts::schemas::get_core_gts_schemas`
    query: Option<String>,
}

impl SourceArgs {
    pub fn run(self) -> anyhow::Result<()> {
        gears_cli_core::source::SourceParams::from(self).run()
    }
}

impl From<SourceArgs> for gears_cli_core::source::SourceParams {
    fn from(args: SourceArgs) -> Self {
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
