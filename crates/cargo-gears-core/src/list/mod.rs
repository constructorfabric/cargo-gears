mod gears;
pub mod templates;

pub use gears::{GearsOutput, GearsParams};
pub use templates::TemplatesParams;

#[derive(Debug, Eq, PartialEq)]
pub enum ListCommand {
    Gears(GearsParams),
    Templates(TemplatesParams),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ListParams {
    pub command: ListCommand,
}

impl ListParams {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ListCommand::Gears(args) => args.run(),
            ListCommand::Templates(args) => args.run(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SystemRegistryModule {
    pub module_name: &'static str,
    pub crate_name: &'static str,
}

pub const SYSTEM_REGISTRY_MODULES: &[SystemRegistryModule] = &[
    SystemRegistryModule {
        module_name: "credstore",
        crate_name: "cf-gears-credstore",
    },
    SystemRegistryModule {
        module_name: "file-parser",
        crate_name: "cf-gears-file-parser",
    },
    SystemRegistryModule {
        module_name: "api-gateway",
        crate_name: "cf-gears-api-gateway",
    },
    SystemRegistryModule {
        module_name: "authn-resolver",
        crate_name: "cf-gears-authn-resolver",
    },
    SystemRegistryModule {
        module_name: "static-authn-plugin",
        crate_name: "cf-gears-static-authn-plugin",
    },
    SystemRegistryModule {
        module_name: "authz-resolver",
        crate_name: "cf-gears-authz-resolver",
    },
    SystemRegistryModule {
        module_name: "static-authz-plugin",
        crate_name: "cf-gears-static-authz-plugin",
    },
    SystemRegistryModule {
        module_name: "grpc-hub",
        crate_name: "cf-gears-grpc-hub",
    },
    SystemRegistryModule {
        module_name: "module-orchestrator",
        crate_name: "cf-gears-module-orchestrator",
    },
    SystemRegistryModule {
        module_name: "nodes-registry",
        crate_name: "cf-gears-nodes-registry",
    },
    SystemRegistryModule {
        module_name: "oagw",
        crate_name: "cf-gears-oagw",
    },
    SystemRegistryModule {
        module_name: "single-tenant-tr-plugin",
        crate_name: "cf-gears-single-tenant-tr-plugin",
    },
    SystemRegistryModule {
        module_name: "static-tr-plugin",
        crate_name: "cf-gears-static-tr-plugin",
    },
    SystemRegistryModule {
        module_name: "tenant-resolver",
        crate_name: "cf-gears-tenant-resolver",
    },
    SystemRegistryModule {
        module_name: "types-registry",
        crate_name: "cf-gears-types-registry",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{OutputFormat, Registry};
    use crate::gears_parser::get_module_name_from_crate;
    use crate::gears_parser::test_utils::TempDirExt;
    use tempfile::TempDir;

    /// Scaffolds a temporary Cargo workspace with the given module crates.
    /// Each entry is `(crate_name, module_name)`.
    fn scaffold_workspace(modules: &[(&str, &str)]) -> TempDir {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        let members: Vec<_> = modules
            .iter()
            .map(|(crate_name, _)| format!(r#""{crate_name}""#))
            .collect();
        let members_str = members.join(", ");

        temp_dir.write(
            "Cargo.toml",
            &format!(
                r#"
                [workspace]
                members = [{members_str}]
                resolver = "3"
                "#,
            ),
        );

        for (crate_name, module_name) in modules {
            temp_dir.write(
                &format!("{crate_name}/Cargo.toml"),
                &format!(
                    r#"
                    [package]
                    name = "{crate_name}"
                    version = "0.1.0"
                    edition = "2024"

                    [lib]
                    path = "src/lib.rs"
                    "#,
                ),
            );
            temp_dir.write(&format!("{crate_name}/src/lib.rs"), "pub mod module;");
            temp_dir.write(
                &format!("{crate_name}/src/module.rs"),
                &format!(
                    r#"
                    #[toolkit::gear(name = "{module_name}")]
                    pub struct Module;
                    "#,
                ),
            );
        }

        temp_dir
    }

    #[test]
    fn local_modules_discovers_workspace_modules() {
        let temp_dir = scaffold_workspace(&[("crate-alpha", "alpha"), ("crate-beta", "beta")]);

        let modules = get_module_name_from_crate(Some(temp_dir.path()))
            .expect("module discovery should succeed");
        assert_eq!(modules.len(), 2);
        assert!(
            modules.contains_key("alpha"),
            "should discover 'alpha' module"
        );
        assert!(
            modules.contains_key("beta"),
            "should discover 'beta' module"
        );
    }

    #[test]
    fn local_modules_discovers_annotation_in_any_src_rs_file() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["crate-delta"]
            resolver = "3"
            "#,
        );
        temp_dir.write(
            "crate-delta/Cargo.toml",
            r#"
            [package]
            name = "crate-delta"
            version = "0.1.0"
            edition = "2024"

            [lib]
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write("crate-delta/src/lib.rs", "pub mod gear;");
        temp_dir.write(
            "crate-delta/src/gear.rs",
            r#"
            #[toolkit::gear(name = "delta")]
            pub struct Delta;
            "#,
        );

        let modules = get_module_name_from_crate(Some(temp_dir.path()))
            .expect("module discovery should succeed");
        assert_eq!(modules.len(), 1);
        assert!(
            modules.contains_key("delta"),
            "should discover 'delta' module in src/gear.rs"
        );
    }

    #[test]
    fn local_modules_discovers_annotation_in_nested_src_subdir() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["crate-epsilon"]
            resolver = "3"
            "#,
        );
        temp_dir.write(
            "crate-epsilon/Cargo.toml",
            r#"
            [package]
            name = "crate-epsilon"
            version = "0.1.0"
            edition = "2024"

            [lib]
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write("crate-epsilon/src/lib.rs", "pub mod inner;");
        temp_dir.write(
            "crate-epsilon/src/inner/mod.rs",
            r#"
            #[toolkit::gear(name = "epsilon")]
            pub struct Epsilon;
            "#,
        );

        let modules = get_module_name_from_crate(Some(temp_dir.path()))
            .expect("module discovery should succeed");
        assert_eq!(modules.len(), 1);
        assert!(
            modules.contains_key("epsilon"),
            "should discover 'epsilon' module in src/inner/mod.rs"
        );
    }

    #[test]
    fn local_modules_empty_workspace_finds_none() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["no-module"]
            resolver = "3"
            "#,
        );
        temp_dir.write(
            "no-module/Cargo.toml",
            r#"
            [package]
            name = "no-module"
            version = "0.1.0"
            edition = "2024"

            [lib]
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write("no-module/src/lib.rs", "pub fn hello() {}");

        let modules = get_module_name_from_crate(Some(temp_dir.path()))
            .expect("module discovery should succeed");
        assert!(
            modules.is_empty(),
            "workspace without gears module annotation should find no modules"
        );
    }

    #[test]
    fn list_modules_local_runs_successfully() {
        let temp_dir = scaffold_workspace(&[("crate-gamma", "gamma")]);

        let args = GearsParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: false,
            output: GearsOutput::local(),
            registry: Registry::CratesIo,
            format: OutputFormat::Table,
        };

        args.run().expect("list gears --local should succeed");
    }

    #[test]
    fn list_modules_local_verbose_runs_successfully() {
        let temp_dir = scaffold_workspace(&[("crate-delta", "delta")]);

        let args = GearsParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: true,
            output: GearsOutput::local(),
            registry: Registry::CratesIo,
            format: OutputFormat::Table,
        };

        args.run()
            .expect("list gears --local --verbose should succeed");
    }

    #[test]
    fn list_modules_system_runs_successfully() {
        let args = GearsParams {
            path: None,
            verbose: false,
            output: GearsOutput::system(),
            registry: Registry::CratesIo,
            format: OutputFormat::Table,
        };

        args.run().expect("list gears --system should succeed");
    }

    #[test]
    fn system_modules_registry_is_not_empty() {
        assert!(
            !SYSTEM_REGISTRY_MODULES.is_empty(),
            "system registry should contain at least one module"
        );
    }

    #[test]
    fn list_modules_combines_system_and_local() {
        let temp_dir = scaffold_workspace(&[("crate-one", "one"), ("crate-two", "two")]);

        let args = GearsParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: false,
            output: GearsOutput::all(),
            registry: Registry::CratesIo,
            format: OutputFormat::Table,
        };

        args.run().expect("list modules should succeed");
    }

    #[test]
    fn list_local_modules_verbose_includes_metadata() {
        let temp_dir = scaffold_workspace(&[("crate-echo", "echo")]);

        let modules = get_module_name_from_crate(Some(temp_dir.path()))
            .expect("module discovery should succeed");
        let echo = modules.get("echo").expect("should discover 'echo' module");
        assert_eq!(echo.metadata.package.as_deref(), Some("crate-echo"));
        assert_eq!(echo.metadata.version.as_deref(), Some("0.1.0"));
    }
}
