use crate::common::with_current_dir_for_optional_path;
use crate::module_parser::{ConfigModuleMetadata, get_module_name_from_crate};
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct LocalModulesArgs {
    pub path: Option<PathBuf>,
    pub verbose: bool,
}

impl LocalModulesArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        with_current_dir_for_optional_path(self.path.as_deref(), || {
            let local_modules = get_module_name_from_crate()?;

            println!("Workspace modules:");
            if local_modules.is_empty() {
                println!("  (none)");
            } else {
                let mut entries: Vec<_> = local_modules.iter().collect();
                entries.sort_by_key(|(name, _)| *name);
                for (module_name, module) in entries {
                    println!("  - {module_name}");
                    if self.verbose {
                        print_metadata(&module.metadata);
                    }
                }
            }

            Ok(())
        })
    }
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
