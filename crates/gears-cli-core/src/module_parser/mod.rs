mod config;
mod metadata;
mod module_rs;
mod source;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub use config::*;
pub use metadata::*;
pub use module_rs::{ParsedModule, parse_module_rs_source};
pub use source::{NotFoundError, ResolvedRustPath, extract_reexport_target, resolve_rust_path};
