//! Running `cargo-gears-lints` through `cargo dylint`.
//!
//! A workspace declares `cargo-gears-lints` in `workspace.metadata.dylint.libraries`, like any
//! other Dylint library. The CLI only checks that the entry is pinned to the commit the CLI was
//! released from and then runs `cargo dylint` over the selected packages, so every library the
//! workspace declares is loaded.

use crate::common::cargo_cmd;
use crate::packages::PackageScope;
use anyhow::{Context, Result, bail};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fmt, fs};
use toml_edit::{DocumentMut, InlineTable, Item, Table, TableLike, Value};

/// Repository that hosts `cargo-gears-lints`.
const LINTS_GIT_URL: &str = "https://github.com/constructorfabric/cargo-gears";

/// Location of the `cargo-gears-lints` package inside [`LINTS_GIT_URL`].
const LINTS_PATTERN: &str = "crates/cargo-gears-lints";

/// Package name of the lints, used to recognize local checkouts.
const LINTS_PACKAGE: &str = "cargo-gears-lints";

/// Tag of the commit this CLI was released from.
const LINTS_TAG: &str = concat!("cargo-gears-v", env!("CARGO_PKG_VERSION"));

/// Path of the Dylint metadata table in both [`LibrariesFile`]s.
const DYLINT_TABLE_PATH: [&str; 3] = ["workspace", "metadata", "dylint"];

/// Binaries `cargo dylint` needs: the subcommand itself and the linker `cargo-gears-lints`
/// builds with (see its `.cargo/config.toml`).
const DYLINT_TOOLS: [&str; 2] = ["cargo-dylint", "dylint-link"];

/// Flags the Dylint driver appends to `rustc`'s arguments for workspace packages.
const DYLINT_RUSTFLAGS: &str = "DYLINT_RUSTFLAGS";

/// File that declares the workspace's Dylint libraries.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum LibrariesFile {
    /// The workspace `Cargo.toml`.
    CargoToml,
    /// A `dylint.toml` next to it, which Dylint reads as well.
    DylintToml,
}

impl LibrariesFile {
    /// Name of the file in the workspace root.
    const fn file_name(self) -> &'static str {
        match self {
            Self::CargoToml => "Cargo.toml",
            Self::DylintToml => "dylint.toml",
        }
    }
}

impl fmt::Display for LibrariesFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.file_name())
    }
}

/// How the workspace refers to `cargo-gears-lints`.
#[derive(Debug, Eq, PartialEq)]
enum LintsSource {
    /// Fetched from [`LINTS_GIT_URL`] at [`LINTS_TAG`].
    Pinned,
    /// A local checkout, used while developing the lints.
    Local(PathBuf),
}

/// One `workspace.metadata.dylint.libraries` entry that refers to `cargo-gears-lints`.
#[derive(Debug, Eq, PartialEq)]
enum LintsEntry {
    /// A git entry for [`LINTS_GIT_URL`], with every way it differs from the expected pin.
    Git { problems: Vec<String> },
    /// A path entry for a local checkout.
    Local(PathBuf),
}

/// Run `cargo dylint` with every library the workspace declares, after checking that
/// `cargo-gears-lints` is pinned to [`LINTS_TAG`]. `feature_args` are Cargo's feature flags.
pub(super) fn run(
    workspace_root: &Path,
    skipped_lints: &[String],
    package_scope: &PackageScope,
    locked: bool,
    feature_args: &[String],
) -> Result<()> {
    if let LintsSource::Local(path) = lints_source(workspace_root)? {
        eprintln!(
            "warning: using cargo-gears-lints from {} instead of tag {LINTS_TAG}",
            path.display()
        );
    }

    ensure_tools_installed()?;

    let mut cmd = cargo_cmd()?;
    cmd.args(cargo_dylint_args(
        workspace_root,
        skipped_lints,
        package_scope,
        locked,
        feature_args,
    )?);
    let existing_rustflags = env::var(DYLINT_RUSTFLAGS).ok();
    if let Some(rustflags) = dylint_rustflags(skipped_lints, existing_rustflags.as_deref()) {
        cmd.env(DYLINT_RUSTFLAGS, rustflags);
    }
    cmd.current_dir(workspace_root);

    let status = cmd.status().context("failed to run `cargo dylint`")?;
    if !status.success() {
        bail!("`cargo dylint` failed with exit status {status}");
    }

    Ok(())
}

/// Point the workspace's `cargo-gears-lints` entry at [`LINTS_TAG`], adding the entry when it is
/// missing.
pub fn pin_lints(workspace_root: &Path) -> Result<()> {
    let file = libraries_file(workspace_root)?;
    let path = workspace_root.join(file.file_name());
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let pinned = pin_lints_in(&content, file, workspace_root)
        .with_context(|| format!("failed to pin cargo-gears-lints in {}", path.display()))?;

    if pinned == content {
        println!("cargo-gears-lints is already pinned to {LINTS_TAG} in {file}");
    } else {
        fs::write(&path, pinned).with_context(|| format!("failed to write {}", path.display()))?;
        println!("Pinned cargo-gears-lints to {LINTS_TAG} in {file}");
    }

    Ok(())
}

/// The file that declares the workspace's Dylint libraries: `dylint.toml` when it declares any,
/// and the workspace `Cargo.toml` otherwise.
///
/// Dylint loads libraries from both files and rejects a workspace that declares them in both, so
/// an empty `libraries` array in `dylint.toml` does not override `Cargo.toml`.
fn libraries_file(workspace_root: &Path) -> Result<LibrariesFile> {
    let in_cargo_toml = declares_libraries(workspace_root, LibrariesFile::CargoToml)?;
    let in_dylint_toml = declares_libraries(workspace_root, LibrariesFile::DylintToml)?;

    match (in_cargo_toml, in_dylint_toml) {
        (true, true) => bail!(
            "`workspace.metadata.dylint.libraries` cannot appear in both Cargo.toml and \
             dylint.toml; keep the libraries in one of them"
        ),
        (false, true) => Ok(LibrariesFile::DylintToml),
        _ => Ok(LibrariesFile::CargoToml),
    }
}

/// Whether `file` in `workspace_root` declares at least one Dylint library.
fn declares_libraries(workspace_root: &Path, file: LibrariesFile) -> Result<bool> {
    let path = workspace_root.join(file.file_name());
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err).with_context(|| format!("failed to read {}", path.display())),
    };

    Ok(match libraries_item(&parse(&content, file)?) {
        None => false,
        Some(Item::Value(Value::Array(array))) => !array.is_empty(),
        // An array of tables has at least one table; any other value is left to `libraries` to
        // reject.
        Some(_) => true,
    })
}

/// Read the workspace's Dylint libraries and check its `cargo-gears-lints` entry.
fn lints_source(workspace_root: &Path) -> Result<LintsSource> {
    let file = libraries_file(workspace_root)?;
    let path = workspace_root.join(file.file_name());
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    lints_source_in(&parse(&content, file)?, file, workspace_root)
}

/// Check that `doc` declares exactly one `cargo-gears-lints` entry, pinned to [`LINTS_TAG`] or
/// pointing at a local checkout.
fn lints_source_in(
    doc: &DocumentMut,
    file: LibrariesFile,
    workspace_root: &Path,
) -> Result<LintsSource> {
    let mut entries = libraries(doc)?
        .into_iter()
        .filter_map(|library| lints_entry(library, workspace_root));

    let Some(entry) = entries.next() else {
        bail!(
            "cargo-gears-lints is not declared in `workspace.metadata.dylint.libraries` of \
             {file}. Run `cargo gears lint --pin-dylint` or add:\n\n{}",
            pinned_snippet()
        );
    };
    if entries.next().is_some() {
        bail!(
            "cargo-gears-lints is declared more than once in \
             `workspace.metadata.dylint.libraries` of {file}; keep a single entry"
        );
    }

    match entry {
        LintsEntry::Local(path) => Ok(LintsSource::Local(path)),
        LintsEntry::Git { problems } if problems.is_empty() => Ok(LintsSource::Pinned),
        LintsEntry::Git { problems } => bail!(
            "cargo-gears-lints in {file} must be pinned to tag {LINTS_TAG}, which this CLI was \
             released from:\n{}\nRun `cargo gears lint --pin-dylint` to fix it.",
            problems
                .iter()
                .map(|problem| format!("  - {problem}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

/// Recognize a library entry that refers to `cargo-gears-lints`.
fn lints_entry(library: &dyn TableLike, workspace_root: &Path) -> Option<LintsEntry> {
    let patterns = patterns(library);

    if let Some(git) = string(library, "git") {
        if !is_lints_repo(git) {
            return None;
        }

        let mut problems = Vec::new();
        match string(library, "tag") {
            Some(LINTS_TAG) => {}
            Some(tag) => problems.push(format!("`tag` is `{tag}`, expected `{LINTS_TAG}`")),
            None => problems.push(format!("`tag` is missing, expected `{LINTS_TAG}`")),
        }
        for key in ["branch", "rev"] {
            if library.contains_key(key) {
                problems.push(format!("`{key}` must not be set"));
            }
        }
        if patterns != [LINTS_PATTERN] {
            problems.push(format!("`pattern` must be `{LINTS_PATTERN}`"));
        }

        return Some(LintsEntry::Git { problems });
    }

    let root = workspace_root.join(string(library, "path")?);
    let candidates = if patterns.is_empty() {
        vec![root]
    } else {
        patterns.iter().map(|pattern| root.join(pattern)).collect()
    };
    candidates
        .into_iter()
        .find(|dir| package_name(dir).as_deref() == Some(LINTS_PACKAGE))
        .map(LintsEntry::Local)
}

/// Rewrite `content` so it declares `cargo-gears-lints` pinned to [`LINTS_TAG`], keeping the
/// other libraries and keys untouched.
fn pin_lints_in(content: &str, file: LibrariesFile, workspace_root: &Path) -> Result<String> {
    let mut doc = parse(content, file)?;

    if file == LibrariesFile::CargoToml && !doc.contains_key("workspace") {
        bail!("Cargo.toml has no `[workspace]` table; run this from the workspace root");
    }

    let mut dylint = doc.as_table_mut() as &mut dyn TableLike;
    for key in DYLINT_TABLE_PATH {
        dylint = dylint
            .entry(key)
            .or_insert_with(|| {
                let mut table = Table::new();
                table.set_implicit(true);
                Item::Table(table)
            })
            .as_table_like_mut()
            .with_context(|| format!("`{key}` must be a table"))?;
    }

    let libraries = dylint
        .entry("libraries")
        .or_insert_with(|| Item::Value(Value::Array(toml_edit::Array::new())));

    match libraries {
        Item::Value(Value::Array(array)) => {
            let ours = (0..array.len())
                .filter(|&index| {
                    array
                        .get(index)
                        .and_then(Value::as_inline_table)
                        .is_some_and(|library| lints_entry(library, workspace_root).is_some())
                })
                .collect::<Vec<_>>();

            if let Some((&first, rest)) = ours.split_first() {
                for &index in rest.iter().rev() {
                    array.remove(index);
                }
                if let Some(library) = array.get_mut(first).and_then(Value::as_inline_table_mut) {
                    pin_entry(library);
                }
            } else {
                let mut library = InlineTable::new();
                pin_entry(&mut library);
                array.push(library);
            }
        }
        Item::ArrayOfTables(tables) => {
            let ours = tables
                .iter()
                .enumerate()
                .filter(|(_, library)| lints_entry(*library, workspace_root).is_some())
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            if let Some((&first, rest)) = ours.split_first() {
                for &index in rest.iter().rev() {
                    tables.remove(index);
                }
                if let Some(library) = tables.get_mut(first) {
                    pin_entry(library);
                }
            } else {
                let mut library = Table::new();
                pin_entry(&mut library);
                tables.push(library);
            }
        }
        _ => bail!("`workspace.metadata.dylint.libraries` must be an array"),
    }

    Ok(doc.to_string())
}

/// Turn a library entry into the expected `cargo-gears-lints` pin.
fn pin_entry(library: &mut dyn TableLike) {
    for key in ["path", "branch", "rev"] {
        library.remove(key);
    }
    library.insert("git", toml_edit::value(LINTS_GIT_URL));
    library.insert("tag", toml_edit::value(LINTS_TAG));
    library.insert("pattern", toml_edit::value(LINTS_PATTERN));
}

/// The entry `pin_lints` writes, as a snippet for error messages.
fn pinned_snippet() -> String {
    format!(
        "[workspace.metadata.dylint]\nlibraries = [\n    {{ git = \"{LINTS_GIT_URL}\", tag = \
         \"{LINTS_TAG}\", pattern = \"{LINTS_PATTERN}\" }},\n]"
    )
}

/// Parse `content` of `file`, keeping its formatting for edits.
fn parse(content: &str, file: LibrariesFile) -> Result<DocumentMut> {
    content
        .parse()
        .with_context(|| format!("failed to parse {file}"))
}

/// The `workspace.metadata.dylint.libraries` item, if `doc` declares it.
fn libraries_item(doc: &DocumentMut) -> Option<&Item> {
    DYLINT_TABLE_PATH
        .iter()
        .try_fold(doc.as_item(), |item, key| item.get(key))?
        .get("libraries")
}

/// The library entries of `doc`, written either as an array of inline tables or as an array of
/// tables. Entries that are not tables are left to Dylint to reject.
fn libraries(doc: &DocumentMut) -> Result<Vec<&dyn TableLike>> {
    Ok(match libraries_item(doc) {
        None => Vec::new(),
        Some(Item::Value(Value::Array(array))) => array
            .iter()
            .filter_map(|library| library.as_inline_table().map(|t| t as &dyn TableLike))
            .collect(),
        Some(Item::ArrayOfTables(tables)) => tables.iter().map(|t| t as &dyn TableLike).collect(),
        Some(_) => bail!("`workspace.metadata.dylint.libraries` must be an array"),
    })
}

/// The string value of `key` in a library entry.
fn string<'a>(library: &'a dyn TableLike, key: &str) -> Option<&'a str> {
    library.get(key)?.as_str()
}

/// Dylint accepts `pattern` as a single string or an array of strings.
fn patterns(library: &dyn TableLike) -> Vec<&str> {
    let Some(item) = library.get("pattern") else {
        return Vec::new();
    };
    if let Some(pattern) = item.as_str() {
        return vec![pattern];
    }
    item.as_array()
        .map(|array| array.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// Whether `git` names [`LINTS_GIT_URL`], with or without a trailing `/` or `.git`.
fn is_lints_repo(git: &str) -> bool {
    git.trim_end_matches('/').trim_end_matches(".git") == LINTS_GIT_URL
}

/// The package name of the `Cargo.toml` in `dir`, if it can be read.
fn package_name(dir: &Path) -> Option<String> {
    let manifest = fs::read_to_string(dir.join("Cargo.toml")).ok()?;
    let doc = manifest.parse::<DocumentMut>().ok()?;
    doc.get("package")?.get("name")?.as_str().map(str::to_owned)
}

/// Install whichever of [`DYLINT_TOOLS`] cannot be found.
fn ensure_tools_installed() -> Result<()> {
    let mut missing = Vec::new();
    for tool in DYLINT_TOOLS {
        // Only whether the tool can be spawned matters, not its exit status: `dylint-link`
        // forwards its arguments to the system linker.
        match Command::new(tool).arg("--version").output() {
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => missing.push(tool),
            Err(err) => return Err(err).with_context(|| format!("failed to run `{tool}`")),
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    eprintln!("Installing {}...", missing.join(", "));
    let status = cargo_cmd()?
        .args(["install", "--locked"])
        .args(&missing)
        .status()
        .context("failed to run `cargo install`")?;
    if !status.success() {
        bail!(
            "`cargo install {}` failed with exit status {status}",
            missing.join(" ")
        );
    }

    Ok(())
}

/// Arguments for `cargo`: the `dylint` subcommand, its manifest and package scope, and the
/// arguments it forwards to `cargo check`.
///
/// Without `--manifest-path`, Dylint only warns when `cargo metadata` fails and can then succeed
/// without loading any library. It has to come before `--` for Dylint to read the metadata.
fn cargo_dylint_args(
    workspace_root: &Path,
    skipped_lints: &[String],
    package_scope: &PackageScope,
    locked: bool,
    feature_args: &[String],
) -> Result<Vec<String>> {
    let manifest_path = workspace_root.join(LibrariesFile::CargoToml.file_name());
    let manifest_path = manifest_path
        .to_str()
        .with_context(|| format!("{} is not valid UTF-8", manifest_path.display()))?;
    let mut args = vec![
        "dylint".to_owned(),
        "--all".to_owned(),
        "--manifest-path".to_owned(),
        manifest_path.to_owned(),
    ];
    match package_scope {
        PackageScope::Workspace => args.push("--workspace".to_owned()),
        PackageScope::Selected(packages) => {
            for package in packages {
                args.extend(["--package".to_owned(), package.clone()]);
            }
        }
    }

    let check_args = cargo_check_args(skipped_lints, locked, feature_args)?;
    if !check_args.is_empty() {
        args.push("--".to_owned());
        args.extend(check_args);
    }

    Ok(args)
}

/// Arguments `cargo dylint` forwards to `cargo check`.
///
/// Skips go into `build.rustflags`, which is part of Cargo's fingerprint, so changing them
/// re-checks the workspace. Cargo ignores it when `RUSTFLAGS` is set, which
/// [`dylint_rustflags`] covers.
fn cargo_check_args(
    skipped_lints: &[String],
    locked: bool,
    feature_args: &[String],
) -> Result<Vec<String>> {
    let mut args = feature_args.to_vec();

    if !skipped_lints.is_empty() {
        let rustflags = skipped_lints
            .iter()
            .flat_map(|lint| ["-A".to_owned(), lint.clone()])
            .collect::<Vec<_>>();
        let rustflags =
            serde_json::to_string(&rustflags).context("failed to encode dylint skips")?;
        args.extend([
            "--config".to_owned(),
            format!("build.rustflags={rustflags}"),
        ]);
    }

    if locked {
        args.push("--locked".to_owned());
    }

    Ok(args)
}

/// `DYLINT_RUSTFLAGS` with the skips appended to `existing`, or `None` without skips.
///
/// The driver appends these flags after every other `rustc` argument, so the skips also hold when
/// `RUSTFLAGS` replaces `build.rustflags` or denies the skipped lints. The driver does not track
/// this variable, which is why [`cargo_check_args`] passes the skips too.
fn dylint_rustflags(skipped_lints: &[String], existing: Option<&str>) -> Option<String> {
    if skipped_lints.is_empty() {
        return None;
    }

    let skips = skipped_lints
        .iter()
        .map(|lint| format!("-A {lint}"))
        .collect::<Vec<_>>()
        .join(" ");
    Some(match existing.map(str::trim) {
        Some(existing) if !existing.is_empty() => format!("{existing} {skips}"),
        _ => skips,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        LINTS_TAG, LibrariesFile, LintsSource, cargo_check_args, cargo_dylint_args,
        dylint_rustflags, lints_source, lints_source_in, pin_lints, pin_lints_in, run,
    };
    use crate::packages::PackageScope;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const WORKSPACE: &str = "[workspace]\nmembers = []\n";

    fn pinned_entry() -> String {
        format!(
            "{{ git = \"https://github.com/constructorfabric/cargo-gears\", tag = \"{LINTS_TAG}\", \
             pattern = \"crates/cargo-gears-lints\" }}"
        )
    }

    fn source(content: &str, root: &Path) -> anyhow::Result<LintsSource> {
        lints_source_in(
            &content.parse().expect("valid toml"),
            LibrariesFile::CargoToml,
            root,
        )
    }

    fn local_lints(temp: &TempDir) {
        let dir = temp.path().join("lints/crates/cargo-gears-lints");
        fs::create_dir_all(&dir).expect("create lints dir");
        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"cargo-gears-lints\"\n",
        )
        .expect("write lints manifest");
    }

    #[test]
    fn accepts_pinned_entry_next_to_other_libraries() {
        let content = format!(
            "{WORKSPACE}\n[workspace.metadata.dylint]\nlibraries = [\n    {{ git = \"https://example.com/other\" }},\n    {},\n]\n",
            pinned_entry()
        );
        assert_eq!(
            source(&content, Path::new(".")).expect("pinned"),
            LintsSource::Pinned
        );
    }

    #[test]
    fn accepts_git_url_with_suffix_and_array_of_tables() {
        let content = format!(
            "{WORKSPACE}\n[[workspace.metadata.dylint.libraries]]\ngit = \"https://github.com/constructorfabric/cargo-gears.git\"\ntag = \"{LINTS_TAG}\"\npattern = [\"crates/cargo-gears-lints\"]\n"
        );
        assert_eq!(
            source(&content, Path::new(".")).expect("pinned"),
            LintsSource::Pinned
        );
    }

    #[test]
    fn rejects_missing_entry() {
        let err = source(WORKSPACE, Path::new(".")).expect_err("missing");
        assert!(err.to_string().contains("--pin-dylint"), "{err}");
    }

    #[test]
    fn rejects_wrong_tag_and_branch() {
        let content = format!(
            "{WORKSPACE}\n[workspace.metadata.dylint]\nlibraries = [{{ git = \"https://github.com/constructorfabric/cargo-gears\", tag = \"cargo-gears-v0.0.1\", branch = \"main\", pattern = \"crates/cargo-gears-lints\" }}]\n"
        );
        let err = source(&content, Path::new(".")).expect_err("wrong tag");
        let err = err.to_string();
        assert!(err.contains("`tag` is `cargo-gears-v0.0.1`"), "{err}");
        assert!(err.contains("`branch` must not be set"), "{err}");
    }

    #[test]
    fn rejects_duplicate_entries() {
        let content = format!(
            "{WORKSPACE}\n[workspace.metadata.dylint]\nlibraries = [{0}, {0}]\n",
            pinned_entry()
        );
        let err = source(&content, Path::new(".")).expect_err("duplicate");
        assert!(err.to_string().contains("more than once"), "{err}");
    }

    #[test]
    fn accepts_local_checkout() {
        let temp = TempDir::new().expect("temp dir");
        local_lints(&temp);
        let content = format!(
            "{WORKSPACE}\n[workspace.metadata.dylint]\nlibraries = [{{ path = \"lints\", pattern = \"crates/cargo-gears-lints\" }}]\n"
        );
        assert_eq!(
            source(&content, temp.path()).expect("local"),
            LintsSource::Local(temp.path().join("lints/crates/cargo-gears-lints"))
        );
    }

    #[test]
    fn pin_adds_entry_to_workspace_without_metadata() {
        let pinned =
            pin_lints_in(WORKSPACE, LibrariesFile::CargoToml, Path::new(".")).expect("pin");
        assert_eq!(
            source(&pinned, Path::new(".")).expect("pinned"),
            LintsSource::Pinned
        );
        assert!(!pinned.contains("[workspace.metadata]\n"), "{pinned}");
    }

    #[test]
    fn pin_replaces_outdated_and_local_entries_and_keeps_others() {
        let temp = TempDir::new().expect("temp dir");
        local_lints(&temp);
        let content = format!(
            "{WORKSPACE}\n[workspace.metadata.dylint]\nlibraries = [\n    {{ git = \"https://example.com/other\" }},\n    {{ git = \"https://github.com/constructorfabric/cargo-gears\", branch = \"main\" }},\n    {{ path = \"lints\", pattern = \"crates/cargo-gears-lints\" }},\n]\n"
        );
        let pinned = pin_lints_in(&content, LibrariesFile::CargoToml, temp.path()).expect("pin");

        assert_eq!(
            source(&pinned, temp.path()).expect("pinned"),
            LintsSource::Pinned
        );
        assert!(pinned.contains("https://example.com/other"), "{pinned}");
        assert!(!pinned.contains("branch"), "{pinned}");
        assert!(!pinned.contains("path ="), "{pinned}");
    }

    #[test]
    fn pin_is_idempotent() {
        let once = pin_lints_in(WORKSPACE, LibrariesFile::CargoToml, Path::new(".")).expect("pin");
        let twice = pin_lints_in(&once, LibrariesFile::CargoToml, Path::new(".")).expect("pin");
        assert_eq!(once, twice);
    }

    #[test]
    fn pin_requires_workspace_cargo_toml() {
        let err = pin_lints_in(
            "[package]\nname = \"a\"\n",
            LibrariesFile::CargoToml,
            Path::new("."),
        )
        .expect_err("not a workspace");
        assert!(err.to_string().contains("[workspace]"), "{err}");
        pin_lints_in("", LibrariesFile::DylintToml, Path::new("."))
            .expect("dylint.toml needs no workspace table");
    }

    #[test]
    fn skip_list_is_converted_to_cargo_rustflags_config() {
        let args = cargo_check_args(
            &[
                "de0301_no_infra_in_domain".to_owned(),
                "de1302_error_from_to_string".to_owned(),
            ],
            false,
            &[],
        )
        .expect("skip args should encode");

        assert_eq!(
            args,
            vec![
                "--config".to_owned(),
                "build.rustflags=[\"-A\",\"de0301_no_infra_in_domain\",\"-A\",\"de1302_error_from_to_string\"]"
                    .to_owned(),
            ]
        );
    }

    fn manifest_path(root: &Path) -> String {
        root.join("Cargo.toml")
            .to_str()
            .expect("utf-8 path")
            .to_owned()
    }

    #[test]
    fn workspace_scope_runs_all_libraries_on_the_workspace() {
        let root = Path::new("ws");
        assert_eq!(
            cargo_dylint_args(root, &[], &PackageScope::Workspace, false, &[]).expect("args"),
            [
                "dylint",
                "--all",
                "--manifest-path",
                &manifest_path(root),
                "--workspace"
            ]
        );
    }

    #[test]
    fn package_scope_features_skips_and_locked_are_forwarded() {
        let root = Path::new("ws");
        let args = cargo_dylint_args(
            root,
            &["de1301_no_print_macros".to_owned()],
            &PackageScope::Selected(vec!["crate-a".to_owned(), "crate-b".to_owned()]),
            true,
            &["--features".to_owned(), "otel".to_owned()],
        )
        .expect("dylint arguments should encode");

        assert_eq!(
            args,
            [
                "dylint",
                "--all",
                "--manifest-path",
                &manifest_path(root),
                "--package",
                "crate-a",
                "--package",
                "crate-b",
                "--",
                "--features",
                "otel",
                "--config",
                "build.rustflags=[\"-A\",\"de1301_no_print_macros\"]",
                "--locked",
            ]
        );
    }

    #[test]
    fn skips_are_appended_to_dylint_rustflags() {
        let skips = [
            "de0301_no_infra_in_domain".to_owned(),
            "de1301_no_print_macros".to_owned(),
        ];

        assert_eq!(dylint_rustflags(&[], Some("-D warnings")), None);
        assert_eq!(
            dylint_rustflags(&skips, None).as_deref(),
            Some("-A de0301_no_infra_in_domain -A de1301_no_print_macros")
        );
        assert_eq!(
            dylint_rustflags(&skips, Some(" ")).as_deref(),
            Some("-A de0301_no_infra_in_domain -A de1301_no_print_macros")
        );
        assert_eq!(
            dylint_rustflags(&skips, Some("-D warnings")).as_deref(),
            Some("-D warnings -A de0301_no_infra_in_domain -A de1301_no_print_macros")
        );
    }

    fn workspace_with_dylint_toml(cargo_libraries: &str, dylint_libraries: &str) -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("Cargo.toml"),
            format!("{WORKSPACE}\n[workspace.metadata.dylint]\nlibraries = [{cargo_libraries}]\n"),
        )
        .expect("write Cargo.toml");
        fs::write(
            temp.path().join("dylint.toml"),
            format!("[workspace.metadata.dylint]\nlibraries = [{dylint_libraries}]\n"),
        )
        .expect("write dylint.toml");
        temp
    }

    fn read(temp: &TempDir, file: &str) -> String {
        fs::read_to_string(temp.path().join(file)).expect("read file")
    }

    #[test]
    fn empty_dylint_toml_libraries_do_not_override_cargo_toml() {
        let temp = workspace_with_dylint_toml("{ git = \"https://example.com/other\" }", "");

        pin_lints(temp.path()).expect("pin");

        assert_eq!(
            read(&temp, "dylint.toml"),
            "[workspace.metadata.dylint]\nlibraries = []\n"
        );
        assert!(read(&temp, "Cargo.toml").contains(LINTS_TAG));
        assert_eq!(
            lints_source(temp.path()).expect("pinned"),
            LintsSource::Pinned
        );
    }

    #[test]
    fn libraries_in_dylint_toml_are_pinned_there() {
        let temp = workspace_with_dylint_toml("", "{ git = \"https://example.com/other\" }");

        pin_lints(temp.path()).expect("pin");

        assert!(!read(&temp, "Cargo.toml").contains(LINTS_TAG));
        assert!(read(&temp, "dylint.toml").contains(LINTS_TAG));
        assert_eq!(
            lints_source(temp.path()).expect("pinned"),
            LintsSource::Pinned
        );
    }

    #[test]
    fn libraries_in_both_files_are_rejected() {
        let temp =
            workspace_with_dylint_toml(&pinned_entry(), "{ git = \"https://example.com/other\" }");
        let cargo_toml = read(&temp, "Cargo.toml");
        let dylint_toml = read(&temp, "dylint.toml");

        let err = pin_lints(temp.path()).expect_err("conflicting libraries");
        assert!(err.to_string().contains("cannot appear in both"), "{err}");
        let err = lints_source(temp.path()).expect_err("conflicting libraries");
        assert!(err.to_string().contains("cannot appear in both"), "{err}");
        assert_eq!(read(&temp, "Cargo.toml"), cargo_toml);
        assert_eq!(read(&temp, "dylint.toml"), dylint_toml);
    }

    #[test]
    #[ignore = "needs `cargo-dylint` and `dylint-link`, and installs them when missing"]
    fn unloadable_workspace_metadata_fails_the_run() {
        let temp = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp.path().join("a/src")).expect("create package dir");
        fs::write(temp.path().join("a/src/lib.rs"), "").expect("write lib.rs");
        fs::write(
            temp.path().join("a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n\
             [dependencies]\nmissing = { path = \"../missing\" }\n",
        )
        .expect("write package manifest");
        fs::write(
            temp.path().join("Cargo.toml"),
            format!(
                "[workspace]\nmembers = [\"a\"]\nresolver = \"3\"\n\n\
                 [workspace.metadata.dylint]\nlibraries = [{}]\n",
                pinned_entry()
            ),
        )
        .expect("write workspace manifest");

        run(temp.path(), &[], &PackageScope::Workspace, false, &[])
            .expect_err("a workspace whose metadata cannot be loaded must fail the run");
    }
}
