use anyhow::{Context, Result};
use guppy::MetadataCommand;
use guppy::graph::DependencyDirection;
use std::collections::BTreeSet;
use std::path::Path;

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
}
