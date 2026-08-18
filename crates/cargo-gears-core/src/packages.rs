use anyhow::{Context, Result};
use guppy::MetadataCommand;
use guppy::graph::DependencyDirection;
use std::collections::BTreeSet;
use std::path::Path;

use guppy::graph::PackageGraph;

fn build_graph(workspace_root: &Path) -> Result<PackageGraph> {
    MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .build_graph()
        .context("failed to build package graph")
}

/// Return all workspace package names, sorted and de-duplicated.
pub fn all_workspace_packages(workspace_root: &Path) -> Result<Vec<String>> {
    let graph = build_graph(workspace_root)?;
    let mut names: Vec<String> = graph
        .packages()
        .filter(guppy::graph::PackageMetadata::in_workspace)
        .map(|pkg| pkg.name().to_owned())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

/// Discover all workspace packages whose manifest path is inside any of the
/// given `scope_dirs`.
pub fn discover_packages(workspace_root: &Path, scope_dirs: &[&Path]) -> Result<Vec<String>> {
    let graph = build_graph(workspace_root)?;

    let mut invalid_dirs = Vec::new();
    let mut scopes = Vec::new();
    for d in scope_dirs {
        match d.canonicalize() {
            Ok(canonical) => scopes.push(canonical),
            Err(_) => invalid_dirs.push(d.display().to_string()),
        }
    }
    if !invalid_dirs.is_empty() {
        anyhow::bail!(
            "scope directories do not exist: {}",
            invalid_dirs.join(", ")
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
    use super::expand_with_dependents;
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
    fn all_workspace_packages_returns_sorted_names() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let packages = super::all_workspace_packages(temp.path()).expect("all_workspace_packages");
        assert_eq!(packages, vec!["leaf", "mid", "other", "top"]);
    }

    fn write_named_member(root: &Path, dir: &str, name: &str, deps: &str) {
        let member_dir = root.join(dir);
        fs::create_dir_all(member_dir.join("src")).expect("create member dir");
        fs::write(
            member_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{deps}"
            ),
        )
        .expect("write member manifest");
        fs::write(member_dir.join("src/lib.rs"), "").expect("write member lib");
    }

    #[test]
    fn discover_packages_scopes_to_directory() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["gears/alpha", "gears/beta", "libs/gamma"]
"#,
        )
        .expect("write root manifest");

        write_named_member(temp.path(), "gears/alpha", "alpha", "");
        write_named_member(temp.path(), "gears/beta", "beta", "");
        write_named_member(temp.path(), "libs/gamma", "gamma", "");

        let gears_dir = temp.path().join("gears");
        let packages =
            super::discover_packages(temp.path(), &[gears_dir.as_path()]).expect("discover");
        assert_eq!(packages, vec!["alpha", "beta"]);
    }

    #[test]
    fn discover_packages_rejects_all_invalid_scope_dirs() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["alpha"]
"#,
        )
        .expect("write root manifest");
        write_member(temp.path(), "alpha", "");

        let bad = temp.path().join("nonexistent");
        let err = super::discover_packages(temp.path(), &[bad.as_path()])
            .expect_err("should reject invalid scope dir");
        assert!(
            err.to_string().contains("scope directories do not exist"),
            "error was: {err}"
        );
        assert!(
            err.to_string().contains("nonexistent"),
            "error should name the bad dir: {err}"
        );
    }

    #[test]
    fn discover_packages_rejects_mixed_valid_and_invalid_scope_dirs() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["gears/alpha"]
"#,
        )
        .expect("write root manifest");
        write_named_member(temp.path(), "gears/alpha", "alpha", "");

        let valid = temp.path().join("gears");
        let bad = temp.path().join("typo");
        let err = super::discover_packages(temp.path(), &[valid.as_path(), bad.as_path()])
            .expect_err("should reject when any scope dir is invalid");
        assert!(
            err.to_string().contains("typo"),
            "error should name the bad dir: {err}"
        );
    }

    #[test]
    fn packages_scope_dirs_rejects_invalid_dirs() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        let params = crate::list::PackagesParams {
            path: Some(temp.path().to_path_buf()),
            scope_dirs: vec!["nonexistent".to_owned()],
            filter: None,
            include_rdeps: false,
            format: crate::common::OutputFormat::List,
        };

        let err = params
            .run()
            .expect_err("ls packages with invalid scope dir should fail");
        assert!(
            err.to_string().contains("scope directories do not exist"),
            "error was: {err}"
        );
    }

    #[test]
    fn packages_filter_with_include_rdeps_expands_without_scope_dirs() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        // Filter to "leaf" only, then expand with rdeps — should get leaf + mid + top
        let params = crate::list::PackagesParams {
            path: Some(temp.path().to_path_buf()),
            scope_dirs: Vec::new(),
            filter: Some("^leaf$".to_owned()),
            include_rdeps: true,
            format: crate::common::OutputFormat::List,
        };

        params
            .run()
            .expect("ls packages --filter '^leaf$' --include-rdeps should succeed");
    }

    #[test]
    fn packages_filter_without_rdeps_returns_only_matched() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        // Filter to "leaf" only, no rdeps — should get just leaf
        let params = crate::list::PackagesParams {
            path: Some(temp.path().to_path_buf()),
            scope_dirs: Vec::new(),
            filter: Some("^leaf$".to_owned()),
            include_rdeps: false,
            format: crate::common::OutputFormat::List,
        };

        params
            .run()
            .expect("ls packages --filter '^leaf$' should succeed");
    }

    #[test]
    fn packages_filter_no_match_skips_rdeps() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(temp.path());

        // Filter matches nothing — rdeps should not error on empty input
        let params = crate::list::PackagesParams {
            path: Some(temp.path().to_path_buf()),
            scope_dirs: Vec::new(),
            filter: Some("^nonexistent$".to_owned()),
            include_rdeps: true,
            format: crate::common::OutputFormat::List,
        };

        params
            .run()
            .expect("ls packages with no filter match + rdeps should succeed");
    }
}
