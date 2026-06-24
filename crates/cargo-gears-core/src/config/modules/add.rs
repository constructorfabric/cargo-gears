use super::{load_config, save_config, validate_module_name};
use crate::app_config::AppConfig;
use crate::common::PathConfigParams;
use crate::gears_parser::{ConfigModule, ConfigModuleMetadata, get_module_name_from_crate};
use anyhow::{Context, bail};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
pub struct AddParams {
    pub path_config: PathConfigParams,
    /// Module name
    pub module: String,
    /// Dependency name to include in metadata.deps (repeatable)
    pub deps: Vec<String>,
}

impl AddParams {
    pub fn run(&self) -> anyhow::Result<()> {
        self.path_config
            .with_workspace_dir(|workspace_path, config_path| {
                validate_module_name(&self.module)?;

                let mut config = load_config(config_path)?;
                let local_modules = discover_local_modules(workspace_path)?;
                let metadata = build_required_metadata(self, local_modules.get(&self.module))?;

                upsert_module_config(&mut config, self, metadata);

                save_config(config_path, &config)
            })
    }
}

fn upsert_module_config(config: &mut AppConfig, args: &AddParams, incoming: ConfigModuleMetadata) {
    let module_config = config.gears.entry(args.module.clone()).or_default();
    let merged_metadata = if let Some(existing) = module_config.metadata.take() {
        merge_module_metadata(existing, incoming, args)
    } else {
        incoming
    };
    module_config.metadata = Some(merged_metadata);
}

fn merge_module_metadata(
    existing: ConfigModuleMetadata,
    incoming: ConfigModuleMetadata,
    args: &AddParams,
) -> ConfigModuleMetadata {
    let deps = if args.deps.is_empty() {
        if existing.deps.is_empty() {
            incoming.deps
        } else {
            existing.deps
        }
    } else {
        incoming.deps
    };

    ConfigModuleMetadata {
        package: existing.package.or(incoming.package),
        version: existing.version.or(incoming.version),
        features: if existing.features.is_empty() {
            incoming.features
        } else {
            existing.features
        },
        default_features: existing.default_features.or(incoming.default_features),
        path: existing.path.or(incoming.path),
        deps,
        capabilities: if existing.capabilities.is_empty() {
            incoming.capabilities
        } else {
            existing.capabilities
        },
    }
}

fn discover_local_modules(workspace_path: &Path) -> anyhow::Result<HashMap<String, ConfigModule>> {
    get_module_name_from_crate(Some(workspace_path))
        .with_context(|| "failed to discover local modules")
}

fn build_required_metadata(
    args: &AddParams,
    local_module: Option<&ConfigModule>,
) -> anyhow::Result<ConfigModuleMetadata> {
    let mut metadata = local_module.map_or_else(
        || {
            bail!(
                "module '{}' not found locally; only local modules can be added",
                args.module
            )
        },
        |module| Ok(module.metadata.clone()),
    )?;

    // Keep config portable: do not persist local filesystem paths in metadata.
    metadata.path = None;
    if !args.deps.is_empty() {
        metadata.deps.clone_from(&args.deps);
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::{AddParams, build_required_metadata, upsert_module_config};
    use crate::app_config::{AppConfig, ModuleConfig};
    use crate::common::PathConfigParams;
    use crate::gears_parser::{Capability, ConfigModule, ConfigModuleMetadata};
    use std::path::PathBuf;

    #[test]
    fn build_required_metadata_uses_local_package_and_version() {
        let args = AddParams {
            path_config: PathConfigParams {
                path: Some(PathBuf::from(".")),
                config: Some(PathBuf::from(".")),
            },
            module: "demo".to_owned(),
            deps: vec!["authz".to_owned()],
        };
        let local_module = ConfigModule {
            metadata: ConfigModuleMetadata {
                package: Some("cf-demo-local".to_owned()),
                version: Some("0.3.0".to_owned()),
                deps: vec!["tenant-resolver".to_owned()],
                ..ConfigModuleMetadata::default()
            },
        };

        let metadata = build_required_metadata(&args, Some(&local_module)).expect("metadata");
        assert_eq!(metadata.package.as_deref(), Some("cf-demo-local"));
        assert_eq!(metadata.version.as_deref(), Some("0.3.0"));
        assert_eq!(metadata.path, None);
        assert_eq!(metadata.deps, vec!["authz"]);
    }

    #[test]
    fn build_required_metadata_fails_for_unknown_module() {
        let args = AddParams {
            path_config: PathConfigParams {
                path: Some(PathBuf::from(".")),
                config: Some(PathBuf::from(".")),
            },
            module: "demo".to_owned(),
            deps: vec![],
        };

        let Err(err) = build_required_metadata(&args, None) else {
            panic!("should fail");
        };
        assert!(err.to_string().contains("not found locally"));
    }

    #[test]
    fn upsert_module_config_preserves_existing_metadata() {
        let mut config = AppConfig::default();
        config.gears.insert(
            "demo".to_owned(),
            ModuleConfig {
                metadata: Some(ConfigModuleMetadata {
                    package: Some("cf-demo-existing".to_owned()),
                    version: Some("9.9.9".to_owned()),
                    features: vec!["existing-feature".to_owned()],
                    default_features: Some(false),
                    path: Some("modules/existing".to_owned()),
                    deps: vec!["existing-dep".to_owned()],
                    capabilities: vec![Capability::Grpc],
                }),
                ..ModuleConfig::default()
            },
        );

        let args = AddParams {
            path_config: PathConfigParams {
                path: Some(PathBuf::from(".")),
                config: Some(PathBuf::from(".")),
            },
            module: "demo".to_owned(),
            deps: vec![],
        };

        let incoming = ConfigModuleMetadata {
            package: Some("cf-demo-local".to_owned()),
            version: Some("0.3.0".to_owned()),
            features: vec!["local-feature".to_owned()],
            default_features: Some(true),
            path: None,
            deps: vec!["local-dep".to_owned()],
            capabilities: vec![Capability::Rest],
        };

        upsert_module_config(&mut config, &args, incoming);

        let metadata = &config
            .gears
            .get("demo")
            .and_then(|module| module.metadata.as_ref())
            .expect("metadata should be present after upsert");

        assert_eq!(metadata.package.as_deref(), Some("cf-demo-existing"));
        assert_eq!(metadata.version.as_deref(), Some("9.9.9"));
        assert_eq!(metadata.features, vec!["existing-feature"]);
        assert_eq!(metadata.default_features, Some(false));
        assert_eq!(metadata.path.as_deref(), Some("modules/existing"));
        assert_eq!(metadata.deps, vec!["existing-dep"]);
        assert_eq!(metadata.capabilities, vec![Capability::Grpc]);
    }
}
