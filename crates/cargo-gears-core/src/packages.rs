use anyhow::{Context, Result};
use cargo_util_schemas::core::PackageIdSpec;
use globset::Glob;
use guppy::MetadataCommand;
use guppy::graph::DependencyDirection;
use std::collections::BTreeSet;
use std::path::Path;

/// Resolve Cargo package ID specifications to exact workspace package names.
///
/// Exact names are required by tools such as `cargo fmt`, which accepts
/// `--package <NAME>` but not version-qualified Cargo package specifications.
/// Results preserve input order and are de-duplicated.
pub fn resolve_workspace_package_specs(
    workspace_root: &Path,
    specs: &[String],
) -> Result<Vec<String>> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .context("failed to load workspace packages for package selection")?;
    let workspace_members: BTreeSet<_> = metadata.workspace_members.iter().collect();
    let workspace_packages: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| workspace_members.contains(&package.id))
        .collect();
    let mut names = Vec::new();

    for raw_spec in specs {
        if !raw_spec.contains("://")
            && raw_spec
                .chars()
                .any(|character| matches!(character, '*' | '?' | '['))
        {
            let glob_matcher = Glob::new(raw_spec)
                .with_context(|| format!("invalid Cargo package glob `{raw_spec}`"))?
                .compile_matcher();
            let matching_packages: BTreeSet<String> = workspace_packages
                .iter()
                .filter(|package| glob_matcher.is_match(package.name.as_str()))
                .map(|package| package.name.to_string())
                .collect();
            if matching_packages.is_empty() {
                anyhow::bail!(
                    "package specification `{raw_spec}` did not match any workspace package"
                );
            }
            for package in matching_packages {
                if !names.contains(&package) {
                    names.push(package);
                }
            }
            continue;
        }

        let spec = PackageIdSpec::parse(raw_spec)
            .with_context(|| format!("invalid Cargo package specification `{raw_spec}`"))?;
        let mut matches = workspace_packages
            .iter()
            .copied()
            .filter(|package| package.name == spec.name())
            .filter(|package| {
                spec.partial_version()
                    .is_none_or(|version| version.matches(&package.version))
            })
            .filter(|package| package_source_matches_spec(package, &spec))
            .map(|package| package.name.to_string());

        let package = matches.next().with_context(|| {
            format!("package specification `{raw_spec}` did not match any workspace package")
        })?;
        if matches.next().is_some() {
            anyhow::bail!("package specification `{raw_spec}` is ambiguous in this workspace");
        }
        if !names.contains(&package) {
            names.push(package);
        }
    }

    Ok(names)
}

fn package_source_matches_spec(package: &cargo_metadata::Package, spec: &PackageIdSpec) -> bool {
    let Some(url) = spec.url() else {
        return true;
    };
    if url.scheme() != "file" {
        return false;
    }
    let Ok(spec_path) = url.to_file_path() else {
        return false;
    };
    let Some(package_root) = Path::new(package.manifest_path.as_str()).parent() else {
        return false;
    };

    match (
        std::fs::canonicalize(spec_path),
        std::fs::canonicalize(package_root),
    ) {
        (Ok(spec_path), Ok(package_root)) => spec_path == package_root,
        _ => false,
    }
}

/// Resolve local workspace gear names to their Cargo packages.
///
/// A gear includes its annotated package and its conventional nested `sdk/`
/// workspace package, when present. Results are sorted and de-duplicated.
/// Unknown or non-local gears are rejected.
pub fn packages_for_gears(workspace_root: &Path, gears: &[String]) -> Result<Vec<String>> {
    if gears.is_empty() {
        return Ok(Vec::new());
    }

    let local_gears = crate::gears_parser::get_module_name_from_crate(Some(workspace_root))?;
    let requested: BTreeSet<&str> = gears.iter().map(String::as_str).collect();
    let available: BTreeSet<&str> = local_gears.keys().map(String::as_str).collect();
    let missing: Vec<&str> = requested.difference(&available).copied().collect();
    if !missing.is_empty() {
        let available = available.into_iter().collect::<Vec<_>>().join(", ");
        let available = if available.is_empty() {
            "none".to_owned()
        } else {
            available
        };
        anyhow::bail!(
            "gear(s) not found in workspace: {}; available local gears: {available}",
            missing.join(", ")
        );
    }

    let mut package_names = BTreeSet::new();
    let mut sdk_roots = Vec::new();
    for gear in requested {
        let module = local_gears
            .get(gear)
            .with_context(|| format!("local gear `{gear}` disappeared during resolution"))?;
        let package = module
            .metadata
            .package
            .as_ref()
            .with_context(|| format!("local gear `{gear}` has no package metadata"))?;
        package_names.insert(package.clone());
        if let Some(path) = &module.metadata.path {
            let root = std::fs::canonicalize(path)
                .with_context(|| format!("failed to resolve root path for local gear `{gear}`"))?;
            let sdk_root = root.join("sdk");
            if sdk_root.is_dir() {
                sdk_roots.push(std::fs::canonicalize(&sdk_root).with_context(|| {
                    format!("failed to resolve SDK path for local gear `{gear}`")
                })?);
            }
        }
    }

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .context("failed to load workspace packages for gear selection")?;
    let workspace_members: BTreeSet<_> = metadata.workspace_members.iter().collect();

    for package in &metadata.packages {
        if !workspace_members.contains(&package.id) {
            continue;
        }
        let package_root = Path::new(package.manifest_path.as_str())
            .parent()
            .with_context(|| format!("package `{}` has no manifest parent", package.name))?;
        let package_root = std::fs::canonicalize(package_root)
            .with_context(|| format!("failed to resolve root for package `{}`", package.name))?;
        if sdk_roots
            .iter()
            .any(|sdk_root| package_root == sdk_root.as_path())
        {
            package_names.insert(package.name.to_string());
        }
    }

    Ok(package_names.into_iter().collect())
}

/// Expand `packages` to include every workspace crate that (transitively)
/// depends on them — i.e. the reverse-dependency closure.
///
/// The seed packages themselves are always part of the result. The returned
/// list is sorted and de-duplicated. Returns an error if any requested package
/// is not a member of the workspace rooted at `workspace_root`.
///
/// An empty `packages` slice yields an empty result (nothing to expand).
pub fn expand_with_dependents(workspace_root: &Path, packages: &[String]) -> Result<Vec<String>> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }

    let graph = MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .build_graph()
        .context("failed to build package graph for dependent expansion")?;

    let requested: BTreeSet<&str> = packages.iter().map(String::as_str).collect();

    let (seed_ids, found): (Vec<_>, BTreeSet<_>) = graph
        .packages()
        .filter(|pkg| pkg.in_workspace() && requested.contains(pkg.name()))
        .map(|pkg| (pkg.id(), pkg.name()))
        .unzip();

    let missing: Vec<&str> = requested.difference(&found).copied().collect();
    if !missing.is_empty() {
        anyhow::bail!("package(s) not found in workspace: {}", missing.join(", "));
    }

    let set = graph
        .query_reverse(seed_ids)
        .context("failed to compute reverse dependencies")?
        .resolve();

    let mut names: Vec<String> = set
        .packages(DependencyDirection::Forward)
        .filter(guppy::graph::PackageMetadata::in_workspace)
        .map(|pkg| pkg.name().to_owned())
        .collect();
    names.sort();
    names.dedup();

    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::{expand_with_dependents, packages_for_gears, resolve_workspace_package_specs};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Build a small workspace:
    ///   leaf   (no deps)
    ///   mid    -> leaf
    ///   top    -> mid
    ///   other  (independent)
    fn write_workspace(root: &Path) {
        fs::write(
            root.join("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["leaf", "mid", "top", "other"]
"#,
        )
        .expect("write root manifest");

        write_member(root, "leaf", "");
        write_member(root, "mid", "leaf = { path = \"../leaf\" }\n");
        write_member(root, "top", "mid = { path = \"../mid\" }\n");
        write_member(root, "other", "");
    }

    fn write_member(root: &Path, name: &str, deps: &str) {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("src")).expect("create member dir");
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{deps}"
            ),
        )
        .expect("write member manifest");
        fs::write(dir.join("src/lib.rs"), "").expect("write member lib");
    }

    fn write_gear_workspace(root: &Path) {
        fs::write(
            root.join("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["gear", "gear/sdk", "other"]
"#,
        )
        .expect("write root manifest");

        write_member(root, "other", "");
        write_member(root, "gear/sdk", "");
        fs::create_dir_all(root.join("gear/src")).expect("create gear source dir");
        fs::write(
            root.join("gear/Cargo.toml"),
            r#"[package]
name = "cf-file-parser"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("write gear manifest");
        fs::write(
            root.join("gear/src/lib.rs"),
            r#"#[toolkit::gear(name = "file-parser")]
pub struct FileParser;
"#,
        )
        .expect("write gear source");

        let sdk_manifest = root.join("gear/sdk/Cargo.toml");
        let sdk = fs::read_to_string(&sdk_manifest).expect("read sdk manifest");
        fs::write(
            sdk_manifest,
            sdk.replace("name = \"gear/sdk\"", "name = \"cf-file-parser-sdk\""),
        )
        .expect("rewrite sdk package name");
    }

    #[test]
    fn empty_input_returns_empty() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let expanded = expand_with_dependents(temp.path(), &[]).expect("expand");
        assert!(expanded.is_empty());
    }

    #[test]
    fn expands_to_all_dependents_including_seed() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let expanded = expand_with_dependents(temp.path(), &["leaf".to_owned()]).expect("expand");
        assert_eq!(expanded, vec!["leaf", "mid", "top"]);
    }

    #[test]
    fn independent_crate_only_returns_itself() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let expanded = expand_with_dependents(temp.path(), &["other".to_owned()]).expect("expand");
        assert_eq!(expanded, vec!["other"]);
    }

    #[test]
    fn multiple_seeds_are_unioned_and_deduplicated() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let expanded = expand_with_dependents(temp.path(), &["mid".to_owned(), "other".to_owned()])
            .expect("expand");
        assert_eq!(expanded, vec!["mid", "other", "top"]);
    }

    #[test]
    fn package_specs_resolve_to_exact_workspace_names() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let packages = resolve_workspace_package_specs(
            temp.path(),
            &["mid@0.1".to_owned(), "other".to_owned(), "mid".to_owned()],
        )
        .expect("resolve package specs");

        assert_eq!(packages, vec!["mid", "other"]);
    }

    #[test]
    fn package_globs_resolve_to_exact_workspace_names() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let packages =
            resolve_workspace_package_specs(temp.path(), &["*e*".to_owned(), "top".to_owned()])
                .expect("resolve package globs");

        assert_eq!(packages, vec!["leaf", "other", "top"]);
    }

    #[test]
    fn unknown_or_mismatched_package_specs_are_rejected() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        for spec in ["unknown", "mid@2"] {
            let err = resolve_workspace_package_specs(temp.path(), &[spec.to_owned()])
                .expect_err("package spec should fail");
            assert!(err.to_string().contains(spec), "error was: {err}");
        }
    }

    #[test]
    fn gear_selection_includes_implementation_and_nested_sdk_package() {
        let temp = TempDir::new().expect("temp dir");
        write_gear_workspace(temp.path());

        let packages = packages_for_gears(temp.path(), &["file-parser".to_owned()])
            .expect("resolve gear packages");

        assert_eq!(packages, vec!["cf-file-parser", "cf-file-parser-sdk"]);
    }

    #[cfg(unix)]
    #[test]
    fn gear_selection_includes_a_symlinked_sdk_package() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["gear", "gear/sdk"]
"#,
        )
        .expect("write root manifest");
        fs::create_dir_all(temp.path().join("gear/src")).expect("create gear source");
        fs::write(
            temp.path().join("gear/Cargo.toml"),
            r#"[package]
name = "cf-symlinked"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("write gear manifest");
        fs::write(
            temp.path().join("gear/src/lib.rs"),
            r#"#[toolkit::gear(name = "symlinked")]
pub struct Symlinked;
"#,
        )
        .expect("write gear source");
        write_member(temp.path(), "sdk-target", "");
        let sdk_manifest = temp.path().join("sdk-target/Cargo.toml");
        let sdk = fs::read_to_string(&sdk_manifest).expect("read sdk manifest");
        fs::write(
            sdk_manifest,
            sdk.replace("name = \"sdk-target\"", "name = \"cf-symlinked-sdk\""),
        )
        .expect("rewrite sdk package name");
        symlink("../sdk-target", temp.path().join("gear/sdk")).expect("symlink sdk directory");

        let packages = packages_for_gears(temp.path(), &["symlinked".to_owned()])
            .expect("resolve symlinked gear packages");

        assert_eq!(packages, vec!["cf-symlinked", "cf-symlinked-sdk"]);
    }

    #[test]
    fn root_gear_does_not_select_unrelated_nested_workspace_packages() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"[package]
name = "root-gear"
version = "0.1.0"
edition = "2021"

[workspace]
resolver = "2"
members = ["unrelated"]
"#,
        )
        .expect("write root manifest");
        fs::create_dir_all(temp.path().join("src")).expect("create root source");
        fs::write(
            temp.path().join("src/lib.rs"),
            r#"#[toolkit::gear(name = "root")]
pub struct Root;
"#,
        )
        .expect("write root gear source");
        write_member(temp.path(), "unrelated", "");

        let packages =
            packages_for_gears(temp.path(), &["root".to_owned()]).expect("resolve root gear");

        assert_eq!(packages, vec!["root-gear"]);
    }

    #[test]
    fn unknown_gear_lists_available_local_gears() {
        let temp = TempDir::new().expect("temp dir");
        write_gear_workspace(temp.path());

        let err = packages_for_gears(temp.path(), &["unknown".to_owned()])
            .expect_err("unknown gear should error");
        let message = err.to_string();
        assert!(message.contains("unknown"), "error was: {message}");
        assert!(message.contains("file-parser"), "error was: {message}");
    }

    #[test]
    fn empty_gear_selection_returns_empty_without_workspace_metadata() {
        let temp = TempDir::new().expect("temp dir");

        let packages = packages_for_gears(temp.path(), &[]).expect("empty selection");

        assert!(packages.is_empty());
    }

    #[test]
    fn unknown_package_is_rejected() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let err = expand_with_dependents(temp.path(), &["nope".to_owned()])
            .expect_err("unknown package should error");
        assert!(err.to_string().contains("nope"), "error was: {err}");
    }
}
