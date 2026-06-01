mod local_modules;
mod modules;
mod system_modules;

pub use local_modules::LocalModulesParams;
pub use modules::ModulesParams;
pub use system_modules::SystemModulesParams;

#[derive(Debug, Eq, PartialEq)]
pub enum ListCommand {
    Modules(ModulesParams),
    LocalModules(LocalModulesParams),
    SystemModules(SystemModulesParams),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ListParams {
    pub command: ListCommand,
}

impl ListParams {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ListCommand::Modules(args) => args.run(),
            ListCommand::LocalModules(args) => args.run(),
            ListCommand::SystemModules(args) => args.run(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{OutputFormat, Registry};
    use crate::config::modules::SYSTEM_REGISTRY_MODULES;
    use crate::module_parser::get_module_name_from_crate;
    use crate::module_parser::test_utils::TempDirExt;
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
                    #[module(name = "{module_name}")]
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
            "workspace without module.rs should find no modules"
        );
    }

    #[test]
    fn list_local_modules_runs_successfully() {
        let temp_dir = scaffold_workspace(&[("crate-gamma", "gamma")]);

        let args = LocalModulesParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: false,
            format: OutputFormat::Table,
        };

        args.run().expect("list local-modules should succeed");
    }

    #[test]
    fn list_local_modules_verbose_runs_successfully() {
        let temp_dir = scaffold_workspace(&[("crate-delta", "delta")]);

        let args = LocalModulesParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: true,
            format: OutputFormat::Table,
        };

        args.run()
            .expect("list local-modules --verbose should succeed");
    }

    #[test]
    fn list_system_modules_runs_successfully() {
        let args = SystemModulesParams {
            verbose: false,
            registry: Registry::CratesIo,
            format: OutputFormat::Table,
        };

        args.run().expect("list system-modules should succeed");
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

        let args = ModulesParams {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: false,
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
