use super::{load_config, save_config, validate_name};
use crate::common::PathConfigArgs;
use std::path::PathBuf;

pub mod add;
pub mod db;
pub mod list;
pub mod remove;

#[derive(Clone, Copy)]
pub(super) struct SystemRegistryModule {
    pub module_name: &'static str,
    pub crate_name: &'static str,
}

pub(super) const SYSTEM_REGISTRY_MODULES: &[SystemRegistryModule] = &[
    SystemRegistryModule {
        module_name: "credstore",
        crate_name: "cf-credstore",
    },
    SystemRegistryModule {
        module_name: "file-parser",
        crate_name: "cf-file-parser",
    },
    SystemRegistryModule {
        module_name: "api-gateway",
        crate_name: "cf-api-gateway",
    },
    SystemRegistryModule {
        module_name: "authn-resolver",
        crate_name: "cf-authn-resolver",
    },
    SystemRegistryModule {
        module_name: "static-authn-plugin",
        crate_name: "cf-static-authn-plugin",
    },
    SystemRegistryModule {
        module_name: "authz-resolver",
        crate_name: "cf-authz-resolver",
    },
    SystemRegistryModule {
        module_name: "static-authz-plugin",
        crate_name: "cf-static-authz-plugin",
    },
    SystemRegistryModule {
        module_name: "grpc-hub",
        crate_name: "cf-grpc-hub",
    },
    SystemRegistryModule {
        module_name: "module-orchestrator",
        crate_name: "cf-module-orchestrator",
    },
    SystemRegistryModule {
        module_name: "nodes-registry",
        crate_name: "cf-nodes-registry",
    },
    SystemRegistryModule {
        module_name: "oagw",
        crate_name: "cf-oagw",
    },
    SystemRegistryModule {
        module_name: "single-tenant-tr-plugin",
        crate_name: "cf-single-tenant-tr-plugin",
    },
    SystemRegistryModule {
        module_name: "static-tr-plugin",
        crate_name: "cf-static-tr-plugin",
    },
    SystemRegistryModule {
        module_name: "tenant-resolver",
        crate_name: "cf-tenant-resolver",
    },
    SystemRegistryModule {
        module_name: "types-registry",
        crate_name: "cf-types-registry",
    },
];

pub struct ModulesArgs {
    pub command: ModulesCommand,
}

pub enum ModulesCommand {
    /// List available system crates
    List(list::ListArgs),
    /// Add or update a module in the modules section (upsert)
    Add(add::AddArgs),
    /// Manage module-level database config
    Db(Box<db::ModuleDbArgs>),
    /// Remove a module from the modules section
    Rm(remove::RemoveArgs),
}

pub(super) struct ModulesContext {
    config_path: PathBuf,
}

impl ModulesArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ModulesCommand::List(args) => args.run(),
            ModulesCommand::Add(args) => args.run(),
            ModulesCommand::Db(args) => args.run(),
            ModulesCommand::Rm(args) => args.run(),
        }
    }
}

pub(super) fn resolve_modules_context(
    path_config: &PathConfigArgs,
) -> anyhow::Result<ModulesContext> {
    Ok(ModulesContext {
        config_path: path_config.resolve_config()?,
    })
}

pub(super) fn validate_module_name(module: &str) -> anyhow::Result<()> {
    validate_name(module, "module")
}
