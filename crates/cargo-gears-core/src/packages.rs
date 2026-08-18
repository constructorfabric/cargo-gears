use anyhow::{Context, Result};
use guppy::MetadataCommand;
use guppy::graph::{DependencyDirection, PackageGraph};
use std::collections::BTreeSet;
use std::path::Path;

/// Return all workspace package names, sorted and de-duplicated.
pub fn all_workspace_packages(workspace_root: &Path) -> Result<Vec<String>> {
    let graph = build_graph(workspace_root)?;
    let mut names: Vec<String> = graph
        .packages()
        .filter(|pkg| pkg.in_workspace())
        .map(|pkg| pkg.name().to_owned())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

/// Discover all workspace packages whose manifest path is inside any of the
/// given `scope_dirs`.
///
/// Each entry in `scope_dirs` should be an absolute path.  Results from all
/// directories are unioned.  The returned list is sorted and de-duplicated.
///
/// Returns an error if the graph cannot be built or no packages match.
pub fn discover_packages(workspace_root: &Path, scope_dirs: &[&Path]) -> Result<Vec<String>> {
    let graph = build_graph(workspace_root)?;

    let scopes: Vec<std::path::PathBuf> = scope_dirs
        .iter()
        .filter_map(|d| {
            d.canonicalize()
                .map_err(|e| eprintln!("warning: cannot canonicalize {}: {e}", d.display()))
                .ok()
        })
        .collect();

    if scopes.is_empty() {
        anyhow::bail!(
            "none of the scope directories exist: {}",
            scope_dirs
                .iter()
                .map(|d| d.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let mut names: Vec<String> = graph
        .packages()
        .filter(|pkg| {
            if !pkg.in_workspace() {
                return false;
            }
            let Some(dir) = Path::new(pkg.manifest_path().as_str()).parent() else {
                return false;
            };
            let Ok(dir) = dir.canonicalize() else {
                return false;
            };
            scopes.iter().any(|scope| dir.starts_with(scope))
        })
        .map(|pkg| pkg.name().to_owned())
        .collect();
    names.sort();
    names.dedup();

    if names.is_empty() {
        anyhow::bail!(
            "no workspace packages found under: {}",
            scope_dirs
                .iter()
                .map(|d| d.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    Ok(names)
}

fn build_graph(workspace_root: &Path) -> Result<PackageGraph> {
    MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .build_graph()
        .context("failed to build package graph")
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

    let graph = build_graph(workspace_root)?;

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
    use super::*;
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
    fn unknown_package_is_rejected() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let err = expand_with_dependents(temp.path(), &["nope".to_owned()])
            .expect_err("unknown package should error");
        assert!(err.to_string().contains("nope"), "error was: {err}");
    }

    #[test]
    fn all_workspace_packages_returns_all() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let all = all_workspace_packages(temp.path()).expect("all packages");
        assert_eq!(all, vec!["leaf", "mid", "other", "top"]);
    }

    #[test]
    fn discover_packages_single_dir() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let leaf_dir = temp.path().join("leaf");
        let pkgs = discover_packages(temp.path(), &[leaf_dir.as_path()]).expect("discover");
        assert_eq!(pkgs, vec!["leaf"]);
    }

    #[test]
    fn discover_packages_multiple_dirs() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let leaf_dir = temp.path().join("leaf");
        let other_dir = temp.path().join("other");
        let pkgs =
            discover_packages(temp.path(), &[leaf_dir.as_path(), other_dir.as_path()]).expect("discover");
        assert_eq!(pkgs, vec!["leaf", "other"]);
    }

    #[test]
    fn discover_packages_nonexistent_dir_errors() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let bad = temp.path().join("nonexistent");
        let err = discover_packages(temp.path(), &[bad.as_path()])
            .expect_err("should error for nonexistent dir");
        assert!(
            err.to_string().contains("none of the scope directories"),
            "error was: {err}"
        );
    }
}
