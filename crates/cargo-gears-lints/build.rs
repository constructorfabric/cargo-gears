//! Keeps `crates/cargo-gears/lints.tree` in sync with this directory.
//!
//! This crate isn't published - the CLI fetches it from the repository at its own
//! release tag. release-plz only sees commits that touch files packaged with
//! cargo-gears, so the marker records the git tree id of these sources: every lints
//! change then also changes a cargo-gears file, which bumps its version and lands the
//! commit in its CHANGELOG.md. CI rejects a stale marker
//! (`tools/scripts/lints-tree.sh check`); this script only saves running
//! `make lints-tree` by hand. It does nothing when Dylint builds the lints to run
//! them, so a user's checkout is never touched.
//!
//! It reruns when any hashed file or the marker itself changes. Watching the marker
//! matters: Cargo replays the warnings of the last run, so without a rerun after the
//! marker was rewritten, the "commit it" warning would stick around after the commit.
//! A new top-level file has no watched path yet, so it is picked up once staged,
//! through the repository's index. Watching this directory instead would rerun on
//! every build, as Cargo scans it recursively, `target/` included.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Never fail the build over the marker - CI is what enforces it.
    if let Err(err) = update_marker() {
        println!("cargo:warning=crates/cargo-gears/lints.tree not updated: {err}");
    }
}

fn update_marker() -> Result<(), String> {
    // Dylint is building the lints for `cargo gears lint` (from a clone, or from Cargo's
    // git checkouts for an installed CLI), not for someone changing them.
    println!("cargo:rerun-if-env-changed=DYLINT_BUILDING_LIBRARIES");
    if env::var_os("DYLINT_BUILDING_LIBRARIES").is_some() {
        return Ok(());
    }

    let lints_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?);
    let cli_dir = lints_dir.join("../cargo-gears");
    // Outside a cargo-gears checkout there is no marker to keep in sync. A missing
    // marker inside one is recreated.
    if !cli_dir.join("Cargo.toml").is_file() {
        return Ok(());
    }
    let marker = cli_dir.join("lints.tree");

    let (tree, watched) = lints_tree(&lints_dir)?;
    println!("cargo:rerun-if-changed={}", marker.display());
    for path in watched {
        println!("cargo:rerun-if-changed={path}");
    }

    let current = fs::read_to_string(&marker).unwrap_or_default();
    if current.trim() != tree {
        fs::write(&marker, format!("{tree}\n")).map_err(|e| e.to_string())?;
        println!(
            "cargo:warning=updated crates/cargo-gears/lints.tree to {tree} - commit it with your lints change"
        );
    }
    Ok(())
}

/// The git tree id of the working copy of `lints_dir`, ignored files excluded, and
/// the paths to watch: the top-level entries it covers (Cargo watches directories
/// recursively) and the repository's index.
///
/// It goes through a scratch index, so unlike `make lints-tree` nothing has to be
/// staged first and the user's index is left alone. Once the changes are staged
/// both produce the same id.
fn lints_tree(lints_dir: &Path) -> Result<(String, BTreeSet<String>), String> {
    let index = PathBuf::from(env::var("OUT_DIR").map_err(|e| e.to_string())?).join("lints.index");
    let git = |args: &[&str]| -> Result<String, String> {
        let output = Command::new("git")
            .current_dir(lints_dir)
            .env("GIT_INDEX_FILE", &index)
            .args(args)
            .output()
            .map_err(|e| format!("failed to run git: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "`git {}` failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };

    // "crates/cargo-gears-lints/", relative to the repository root.
    let prefix = git(&["rev-parse", "--show-prefix"])?;
    git(&["read-tree", "--empty"])?;
    git(&["add", "--all", "."])?;
    let tree = git(&["write-tree", &format!("--prefix={prefix}")])?;
    let mut watched: BTreeSet<String> = git(&["ls-files"])?
        .lines()
        .filter_map(|path| path.split('/').next())
        .map(str::to_owned)
        .collect();
    // Not `--git-path index`, which would resolve to the scratch index.
    watched.insert(format!(
        "{}/index",
        git(&["rev-parse", "--absolute-git-dir"])?
    ));
    Ok((tree, watched))
}
