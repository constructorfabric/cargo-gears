use crate::common::OutputFormat;
use crate::module_parser::{ConfigModuleMetadata, get_module_name_from_crate};
use std::fmt::Display;
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct LocalModulesArgs {
    pub path: Option<PathBuf>,
    pub verbose: bool,
    pub format: OutputFormat,
}

impl LocalModulesArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Table => {}
            OutputFormat::Json => {
                todo!("JSON output is not yet implemented for list local-modules")
            }
        }

        print_local_modules(self.verbose, self.path.as_deref())
    }
}

pub(super) fn print_local_modules(
    verbose: bool,
    workspace_dir: Option<&Path>,
) -> anyhow::Result<()> {
    let local_modules = get_module_name_from_crate(workspace_dir)?;

    println!("Workspace modules:");
    if local_modules.is_empty() {
        println!("  (none)");
    } else {
        let mut entries: Vec<_> = local_modules.iter().collect();
        entries.sort_by_key(|(name, _)| *name);
        for (module_name, module) in entries {
            println!("  - {module_name}");
            if verbose {
                print_metadata(&module.metadata);
            }
        }
    }

    Ok(())
}

fn print_metadata(metadata: &ConfigModuleMetadata) {
    print_optional_field("package", metadata.package.as_deref());
    print_optional_field("version", metadata.version.as_deref());
    print_optional_field("path", metadata.path.as_deref());
    print_optional_field("default_features", metadata.default_features.as_ref());

    print_value_list("features", &metadata.features);
    print_value_list("deps", &metadata.deps);
    print_value_list("capabilities", &metadata.capabilities);
}

fn print_optional_field<T: Display>(label: &str, value: Option<T>) {
    if let Some(value) = value {
        println!("      {label}: {value}");
    }
}

fn print_value_list<T: Display>(label: &str, values: &[T]) {
    if values.is_empty() {
        println!("      {label}: (none)");
    } else {
        println!("      {label}:");
        for value in values {
            println!("        - {value}");
        }
    }
}
