use super::config::ConfigModule;
use super::source::{NotFoundError, resolve_rust_path};
use super::{CargoTomlDependencies, CargoTomlDependency};
use anyhow::Context;
use cargo_metadata::{DependencyKind, Package, PackageId, Target};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryMapping {
    pub library_name: String,
    pub package_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMetadataPath {
    pub package_name: String,
    pub library_name: String,
    pub version: String,
    pub manifest_path: PathBuf,
    pub source_path: PathBuf,
    pub source: String,
}

pub fn get_module_name_from_crate(
    workspace_dir: Option<&Path>,
) -> anyhow::Result<HashMap<String, ConfigModule>> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.no_deps();
    if let Some(dir) = workspace_dir {
        cmd.current_dir(dir);
    }
    let res = cmd.exec().context("failed to run cargo metadata")?;
    let mut members = HashMap::new();
    for pkg in res.packages {
        for t in &pkg.targets {
            if is_library_target(t) && !t.name.ends_with("sdk") {
                match super::module_rs::retrieve_gears_module(&pkg, t) {
                    Ok(module) => {
                        members.insert(module.0, module.1);
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                }
            }
        }
    }
    Ok(members)
}

pub fn resolve_source_from_metadata(
    path: &Path,
    query: &str,
) -> anyhow::Result<Option<ResolvedMetadataPath>> {
    // If there's no Cargo.toml, there's no point to run cargo metadata
    if !path.join("Cargo.toml").is_file() {
        return Ok(None);
    }
    let query = RustPathQuery::parse(query)?;
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(path)
        .exec()
        .context("failed to run cargo metadata")?;

    let Some(library_target) = select_library_target(&metadata.packages, &query.package_name)
    else {
        return Ok(None);
    };

    let segments = query
        .segments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let resolved = match resolve_rust_path(&library_target.root_source_path, &segments) {
        Ok(r) => r,
        Err(e) if e.is::<NotFoundError>() => return Ok(None),
        Err(e) => return Err(e),
    };

    Ok(Some(ResolvedMetadataPath {
        package_name: library_target.package_name,
        library_name: library_target.library_name,
        version: library_target.version,
        manifest_path: library_target.manifest_path,
        source_path: resolved.source_path,
        source: resolved.source,
    }))
}

pub fn get_dependencies<S: std::hash::BuildHasher>(
    path: &Path,
    deps: &HashMap<String, String, S>,
) -> anyhow::Result<CargoTomlDependencies> {
    let cargo_metadata::Metadata {
        packages, resolve, ..
    } = cargo_metadata::MetadataCommand::new()
        .current_dir(path)
        .exec()
        .context("failed to run cargo metadata")?;
    let resolve_nodes = resolve.as_ref().map(|resolve| {
        resolve
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node))
            .collect::<HashMap<_, _>>()
    });
    let mut res = CargoTomlDependencies::new();
    for pkg in packages {
        if let Some(name) = deps.get(pkg.name.as_str()) {
            let features = resolve_nodes
                .as_ref()
                .and_then(|nodes| nodes.get(&pkg.id))
                .map(|node| {
                    node.features
                        .iter()
                        .map(ToString::to_string)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default();
            res.insert(
                name.clone(),
                CargoTomlDependency {
                    package: if pkg.name == name {
                        None
                    } else {
                        Some(pkg.name.to_string())
                    },
                    version: Some(pkg.version.to_string()),
                    features,
                    ..Default::default()
                },
            );
        }
    }
    Ok(res)
}

pub fn list_library_mappings_from_metadata(
    path: &Path,
    package_name: &str,
) -> anyhow::Result<Option<Vec<LibraryMapping>>> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(path)
        .exec()
        .context("failed to run cargo metadata")?;

    let Some(library_target) = select_library_target(&metadata.packages, package_name) else {
        return Ok(None);
    };
    let resolve = metadata
        .resolve
        .as_ref()
        .context("cargo metadata did not include a dependency graph")?;
    let Some(node) = resolve
        .nodes
        .iter()
        .find(|node| node.id == library_target.package_id)
    else {
        return Ok(None);
    };

    let packages_by_id = metadata
        .packages
        .iter()
        .map(|package| (package.id.clone(), package))
        .collect::<HashMap<_, _>>();
    let mut mappings = BTreeMap::new();
    mappings.insert(
        library_target.library_name.clone(),
        library_target.package_name.clone(),
    );

    for dep in &node.deps {
        if dep.name.is_empty() || !is_normal_dependency(dep) {
            continue;
        }

        let package = packages_by_id.get(&dep.pkg).with_context(|| {
            format!(
                "dependency '{}' is missing package metadata for package '{}'",
                dep.name, package_name
            )
        })?;

        match mappings.insert(dep.name.clone(), package.name.to_string()) {
            Some(existing) if existing != package.name.as_str() => {
                anyhow::bail!(
                    "library '{}' resolves to conflicting packages '{}' and '{}'",
                    dep.name,
                    existing,
                    package.name
                );
            }
            _ => {}
        }
    }

    Ok(Some(
        mappings
            .into_iter()
            .map(|(library_name, package_name)| LibraryMapping {
                library_name,
                package_name,
            })
            .collect(),
    ))
}

struct RustPathQuery {
    package_name: String,
    segments: Vec<String>,
}

impl RustPathQuery {
    fn parse(value: &str) -> anyhow::Result<Self> {
        let segments: Vec<_> = value
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(str::to_owned)
            .collect();

        let Some((package_name, segments)) = segments.split_first() else {
            anyhow::bail!("query must not be empty");
        };

        Ok(Self {
            package_name: package_name.clone(),
            segments: segments.to_vec(),
        })
    }
}

#[derive(Clone)]
struct LibraryTarget {
    package_id: PackageId,
    package_name: String,
    library_name: String,
    version: String,
    semver_version: cargo_metadata::semver::Version,
    manifest_path: PathBuf,
    root_source_path: PathBuf,
    is_local: bool,
}

fn select_library_target(packages: &[Package], name: &str) -> Option<LibraryTarget> {
    let mut candidates = packages
        .iter()
        .filter(|p| p.name == name)
        .flat_map(|p| {
            p.targets
                .iter()
                .filter(|t| is_library_target(t))
                .map(move |t| to_library_target(p, t))
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .is_local
            .cmp(&left.is_local)
            .then_with(|| right.semver_version.cmp(&left.semver_version))
    });

    candidates.into_iter().next()
}

fn is_library_target(target: &Target) -> bool {
    target.is_lib()
        || target.is_proc_macro()
        || target.is_rlib()
        || target.is_dylib()
        || target.is_cdylib()
        || target.is_staticlib()
}

fn to_library_target(package: &Package, target: &Target) -> LibraryTarget {
    LibraryTarget {
        package_id: package.id.clone(),
        package_name: package.name.to_string(),
        library_name: target.name.clone(),
        version: package.version.to_string(),
        semver_version: package.version.clone(),
        manifest_path: PathBuf::from(&package.manifest_path),
        root_source_path: PathBuf::from(&target.src_path),
        is_local: package.source.is_none(),
    }
}

fn is_normal_dependency(dep: &cargo_metadata::NodeDep) -> bool {
    dep.dep_kinds.is_empty()
        || dep
            .dep_kinds
            .iter()
            .any(|kind| kind.kind == DependencyKind::Normal)
}

#[cfg(test)]
mod tests {
    use super::{
        get_dependencies, list_library_mappings_from_metadata, resolve_source_from_metadata,
    };
    use crate::gears::test_utils::TempDirExt;
    use std::collections::{BTreeSet, HashMap};
    use tempfile::TempDir;

    #[test]
    fn does_not_resolve_using_library_name_from_metadata() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-demo"
            version = "0.2.0"
            edition = "2024"

            [lib]
            name = "demo"
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write(
            "src/lib.rs",
            r"
            pub mod sync;
            ",
        );
        temp_dir.write(
            "src/sync.rs",
            r"
            pub struct Mutex;

            #[cfg(test)]
            mod tests {
                #[test]
                fn hidden() {}
            }
            ",
        );

        let resolved =
            resolve_source_from_metadata(temp_dir.path(), "demo::sync").expect("query should run");

        assert!(resolved.is_none());
    }

    #[test]
    fn resolves_using_package_name_from_metadata() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-demo"
            version = "0.3.0"
            edition = "2024"

            [lib]
            name = "demo"
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write(
            "src/lib.rs",
            r"
            pub mod sync;
            pub struct Root;
            ",
        );
        temp_dir.write(
            "src/sync.rs",
            r"
            pub struct SyncRoot;
            ",
        );

        let resolved = resolve_source_from_metadata(temp_dir.path(), "cf-demo::sync")
            .expect("query should run");
        let resolved = resolved.expect("metadata should resolve query");

        assert_eq!(resolved.library_name, "demo");
        assert!(resolved.source.contains("pub struct SyncRoot;"));
    }

    #[test]
    fn resolves_proc_macro_targets_using_package_name() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-demo-macros"
            version = "0.3.0"
            edition = "2024"

            [lib]
            proc-macro = true
            "#,
        );
        temp_dir.write(
            "src/lib.rs",
            r"
            use proc_macro::TokenStream;

            #[proc_macro_attribute]
            pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
                item
            }
            ",
        );

        let resolved = resolve_source_from_metadata(temp_dir.path(), "cf-demo-macros::module")
            .expect("query should run");
        let resolved = resolved.expect("metadata should resolve proc-macro query");

        assert_eq!(resolved.library_name, "cf_demo_macros");
        assert!(resolved.source.contains("pub fn module"));
    }

    #[test]
    fn lists_library_mappings_using_source_code_dependency_names() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["app", "dep-crate", "cf-helper", "cf-build-helper"]
            resolver = "3"
            "#,
        );
        temp_dir.write(
            "app/Cargo.toml",
            r#"
            [package]
            name = "cf-app"
            version = "0.1.0"
            edition = "2024"

            [lib]
            name = "app"
            path = "src/lib.rs"

            [dependencies]
            helper_alias = { package = "cf-helper", path = "../cf-helper" }
            dep-crate = { path = "../dep-crate" }

            [build-dependencies]
            build_only = { package = "cf-build-helper", path = "../cf-build-helper" }
            "#,
        );
        temp_dir.write(
            "app/src/lib.rs",
            r"
            pub use dep_crate::DepValue;
            pub use helper_alias::helper;
            ",
        );
        temp_dir.write(
            "dep-crate/Cargo.toml",
            r#"
            [package]
            name = "dep-crate"
            version = "0.1.0"
            edition = "2024"

            [lib]
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write(
            "dep-crate/src/lib.rs",
            r"
            pub struct DepValue;
            ",
        );
        temp_dir.write(
            "cf-helper/Cargo.toml",
            r#"
            [package]
            name = "cf-helper"
            version = "0.1.0"
            edition = "2024"

            [lib]
            name = "helper_core"
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write(
            "cf-helper/src/lib.rs",
            r"
            pub fn helper() {}
            ",
        );
        temp_dir.write(
            "cf-build-helper/Cargo.toml",
            r#"
            [package]
            name = "cf-build-helper"
            version = "0.1.0"
            edition = "2024"

            [lib]
            path = "src/lib.rs"
            "#,
        );
        temp_dir.write(
            "cf-build-helper/src/lib.rs",
            r"
            pub fn build_helper() {}
            ",
        );

        let mappings = list_library_mappings_from_metadata(&temp_dir.path().join("app"), "cf-app")
            .expect("metadata query should run")
            .expect("mappings should resolve");

        assert_eq!(
            mappings,
            vec![
                super::LibraryMapping {
                    library_name: "app".to_owned(),
                    package_name: "cf-app".to_owned(),
                },
                super::LibraryMapping {
                    library_name: "dep_crate".to_owned(),
                    package_name: "dep-crate".to_owned(),
                },
                super::LibraryMapping {
                    library_name: "helper_alias".to_owned(),
                    package_name: "cf-helper".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn gets_enabled_features_for_located_packages() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-app"
            version = "0.1.0"
            edition = "2024"

            [dependencies]
            helper_alias = { package = "cf-helper", path = "cf-helper", default-features = false, features = ["grpc", "otel"] }
            "#,
        );
        temp_dir.write(
            "src/lib.rs",
            r"
            pub fn app() {}
            ",
        );
        temp_dir.write(
            "cf-helper/Cargo.toml",
            r#"
            [package]
            name = "cf-helper"
            version = "0.3.0"
            edition = "2024"

            [features]
            default = ["base"]
            base = []
            grpc = []
            otel = []
            "#,
        );
        temp_dir.write(
            "cf-helper/src/lib.rs",
            r"
            pub fn helper() {}
            ",
        );

        let dependency_aliases =
            HashMap::from([("cf-helper".to_owned(), "helper_alias".to_owned())]);
        let dependencies = get_dependencies(temp_dir.path(), &dependency_aliases)
            .expect("metadata should load dependencies");
        let helper = dependencies
            .get("helper_alias")
            .expect("dependency should be present");

        assert_eq!(helper.package.as_deref(), Some("cf-helper"));
        assert_eq!(helper.version.as_deref(), Some("0.3.0"));
        assert_eq!(
            helper.features,
            BTreeSet::from(["grpc".to_owned(), "otel".to_owned()])
        );
    }
}
