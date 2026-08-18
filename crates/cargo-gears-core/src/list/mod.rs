use anyhow::Context;
mod gears;
pub mod templates;

pub use gears::{GearsOutput, GearsParams};
pub use templates::TemplatesParams;

#[derive(Debug, Eq, PartialEq)]
pub enum ListCommand {
    Gears(GearsParams),
    Templates(TemplatesParams),
    Features(FeaturesParams),
    Deps(DepsParams),
    Packages(PackagesParams),
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
            ListCommand::Features(args) => args.run(),
            ListCommand::Deps(args) => args.run(),
            ListCommand::Packages(args) => args.run(),
        }
    }
}

/// Parameters for `cargo gears ls features --manifest <path>`.
///
/// Lists all feature names defined in a Cargo.toml's `[features]` section.
#[derive(Debug, Eq, PartialEq)]
pub struct FeaturesParams {
    pub manifest: std::path::PathBuf,
    pub format: crate::common::OutputFormat,
}

impl FeaturesParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(&self.manifest)
            .with_context(|| format!("cannot read {}", self.manifest.display()))?;
        let doc: toml::Table = content
            .parse()
            .with_context(|| format!("cannot parse {}", self.manifest.display()))?;
        let features = doc
            .get("features")
            .and_then(|v| v.as_table())
            .map(|t| {
                let mut keys: Vec<&str> = t.keys().map(String::as_str).collect();
                keys.sort_unstable();
                keys
            })
            .unwrap_or_default();

        match self.format {
            crate::common::OutputFormat::List | crate::common::OutputFormat::Table => {
                for f in &features {
                    println!("{f}");
                }
            }
            crate::common::OutputFormat::Json => {
                println!("{}", serde_json::to_string(&features)?);
            }
            crate::common::OutputFormat::CargoFlags => {
                println!("{}", features.join(","));
            }
        }
        Ok(())
    }
}

/// Parameters for `cargo gears ls deps --manifest <path>`.
///
/// Lists dependency names from a Cargo.toml's `[dependencies]` section.
/// With `--non-optional`, only non-optional (always-linked) dependencies.
#[derive(Debug, Eq, PartialEq)]
pub struct DepsParams {
    pub manifest: std::path::PathBuf,
    pub non_optional: bool,
    pub format: crate::common::OutputFormat,
}

impl DepsParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(&self.manifest)
            .with_context(|| format!("cannot read {}", self.manifest.display()))?;
        let doc: toml::Table = content
            .parse()
            .with_context(|| format!("cannot parse {}", self.manifest.display()))?;
        let deps_table = doc.get("dependencies").and_then(|v| v.as_table());
        let Some(deps_table) = deps_table else {
            return Ok(());
        };

        let mut names: Vec<String> = Vec::new();
        for (key, value) in deps_table {
            let is_optional = value
                .as_table()
                .and_then(|t| t.get("optional"))
                .and_then(toml::Value::as_bool)
                .unwrap_or(false);
            if self.non_optional && is_optional {
                continue;
            }
            // Use "package" field if present, otherwise the key
            let pkg_name = value
                .as_table()
                .and_then(|t| t.get("package"))
                .and_then(|v| v.as_str())
                .unwrap_or(key);
            names.push(pkg_name.to_owned());
        }
        names.sort();

        match self.format {
            crate::common::OutputFormat::List | crate::common::OutputFormat::Table => {
                for n in &names {
                    println!("{n}");
                }
            }
            crate::common::OutputFormat::Json => {
                println!("{}", serde_json::to_string(&names)?);
            }
            crate::common::OutputFormat::CargoFlags => {
                let flags: Vec<String> = names.iter().map(|n| format!("-p {n}")).collect();
                println!("{}", flags.join(" "));
            }
        }
        Ok(())
    }
}

/// Parameters for `cargo gears ls packages`.
///
/// Lists workspace Cargo packages (all crates, not just annotated gears),
/// with the same filtering options as `ls gears`: `--filter`, `--scope-dirs`,
/// `--include-rdeps`.
#[derive(Debug, Eq, PartialEq)]
pub struct PackagesParams {
    pub path: Option<std::path::PathBuf>,
    pub scope_dirs: Vec<String>,
    pub filter: Option<String>,
    pub include_rdeps: bool,
    pub format: crate::common::OutputFormat,
}

impl PackagesParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_root = crate::common::resolve_workspace_path(self.path.as_deref())?;

        let mut packages = if self.scope_dirs.is_empty() {
            crate::packages::all_workspace_packages(&workspace_root)?
        } else {
            let dirs: Vec<std::path::PathBuf> = self
                .scope_dirs
                .iter()
                .map(|d| {
                    let d = d.trim_end_matches('/');
                    let p = std::path::PathBuf::from(d);
                    if p.is_absolute() {
                        p
                    } else {
                        workspace_root.join(d)
                    }
                })
                .collect();
            let dir_refs: Vec<&std::path::Path> =
                dirs.iter().map(std::path::PathBuf::as_path).collect();
            crate::packages::discover_packages(&workspace_root, &dir_refs)?
        };

        if let Some(pattern) = &self.filter {
            let re = regex::Regex::new(pattern)
                .with_context(|| format!("invalid filter regex: {pattern}"))?;
            packages.retain(|p| re.is_match(p));
        }

        if self.include_rdeps && !packages.is_empty() {
            packages = crate::packages::expand_with_dependents(&workspace_root, &packages)?;
        }

        match self.format {
            crate::common::OutputFormat::List | crate::common::OutputFormat::Table => {
                for p in &packages {
                    println!("{p}");
                }
            }
            crate::common::OutputFormat::Json => {
                println!("{}", serde_json::to_string(&packages)?);
            }
            crate::common::OutputFormat::CargoFlags => {
                let flags: Vec<String> = packages.iter().map(|p| format!("-p {p}")).collect();
                if !flags.is_empty() {
                    println!("{}", flags.join(" "));
                }
            }
        }

        Ok(())
    }
}

use crate::gears_parser::Provision;

#[derive(Clone, Copy)]
pub struct SystemRegistryGear {
    pub gear_name: &'static str,
    pub crate_name: &'static str,
    /// Provider capabilities this gear offers (e.g. `RestHost`, `GrpcHub`).
    /// Used for automatic resolution: when a local gear declares a requirement
    /// like `rest`, `required_provision` maps it to `RestHost`, and the system
    /// gear whose `provides` includes `RestHost` is auto-injected.
    pub provides: &'static [Provision],
}

pub const SYSTEM_REGISTRY_GEARS: &[SystemRegistryGear] = &[
    SystemRegistryGear {
        gear_name: "credstore",
        crate_name: "cf-gears-credstore",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "file-parser",
        crate_name: "cf-gears-file-parser",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "api-gateway",
        crate_name: "cf-gears-api-gateway",
        provides: &[Provision::RestHost],
    },
    SystemRegistryGear {
        gear_name: "authn-resolver",
        crate_name: "cf-gears-authn-resolver",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "static-authn-plugin",
        crate_name: "cf-gears-static-authn-plugin",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "authz-resolver",
        crate_name: "cf-gears-authz-resolver",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "static-authz-plugin",
        crate_name: "cf-gears-static-authz-plugin",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "grpc-hub",
        crate_name: "cf-gears-grpc-hub",
        provides: &[Provision::GrpcHub],
    },
    SystemRegistryGear {
        gear_name: "gear-orchestrator",
        crate_name: "cf-gears-gear-orchestrator",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "nodes-registry",
        crate_name: "cf-gears-nodes-registry",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "oagw",
        crate_name: "cf-gears-oagw",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "single-tenant-tr-plugin",
        crate_name: "cf-gears-single-tenant-tr-plugin",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "static-tr-plugin",
        crate_name: "cf-gears-static-tr-plugin",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "tenant-resolver",
        crate_name: "cf-gears-tenant-resolver",
        provides: &[],
    },
    SystemRegistryGear {
        gear_name: "types-registry",
        crate_name: "cf-gears-types-registry",
        provides: &[],
    },
];

/// Finds the system gear that offers a given [`Provision`].
///
/// Warns if more than one system gear provides the same provision,
/// since only the first match is used.
#[must_use]
pub fn system_gear_for_provision(provision: &Provision) -> Option<&'static SystemRegistryGear> {
    let mut providers = SYSTEM_REGISTRY_GEARS
        .iter()
        .filter(|gear| gear.provides.contains(provision));
    let first = providers.next()?;
    for extra in providers {
        eprintln!(
            "warning: system gear '{}' also provides '{}' but '{}' was already selected",
            extra.gear_name, provision, first.gear_name
        );
    }
    Some(first)
}

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
            filter: None,
            scope_dirs: Vec::new(),
            include_rdeps: false,
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
            filter: None,
            scope_dirs: Vec::new(),
            include_rdeps: false,
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
            filter: None,
            scope_dirs: Vec::new(),
            include_rdeps: false,
        };

        args.run().expect("list gears --system should succeed");
    }

    #[test]
    fn system_modules_registry_is_not_empty() {
        assert!(
            !SYSTEM_REGISTRY_GEARS.is_empty(),
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
            filter: None,
            scope_dirs: Vec::new(),
            include_rdeps: false,
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
