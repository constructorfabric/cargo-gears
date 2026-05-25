use crate::common::Registry;
use crate::config::modules::list::{RegistryMetadata, fetch_all_registry_metadata};
use crate::config::modules::{SYSTEM_REGISTRY_MODULES, SystemRegistryModule};
use anyhow::Context;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq)]
pub struct SystemModulesArgs {
    pub verbose: bool,
    pub registry: Registry,
}

impl SystemModulesArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("System modules:");
        if self.verbose {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("failed to build tokio runtime for registry queries")?;

            let metadata_by_crate = runtime.block_on(fetch_all_registry_metadata(self.registry))?;

            for module in SYSTEM_REGISTRY_MODULES {
                let Some(metadata) = metadata_by_crate.get(module.crate_name) else {
                    anyhow::bail!("missing fetched metadata for '{}'", module.crate_name);
                };

                print_system_registry_metadata(module, metadata);
            }
        } else {
            for module in SYSTEM_REGISTRY_MODULES {
                println!("  - {} (crate: {})", module.module_name, module.crate_name);
            }
        }

        Ok(())
    }
}

fn print_system_registry_metadata(module: &SystemRegistryModule, metadata: &RegistryMetadata) {
    println!("  - {}", module.module_name);
    println!("      crate: {}", module.crate_name);
    println!("      latest_version: {}", metadata.latest_version);
    print_value_list("features", &metadata.features);
    print_value_list("deps", &metadata.deps);
    print_value_list("capabilities", &metadata.capabilities);
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
