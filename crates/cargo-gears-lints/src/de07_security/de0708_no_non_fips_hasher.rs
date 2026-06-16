extern crate rustc_ast;
extern crate rustc_span;

use crate::lint_utils::{filename_str, use_tree_to_strings};
use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::Span;

/// Configuration read from the consuming workspace's `dylint.toml`
/// (under the `[cargo-gears-lints]` table).
#[derive(Default, serde::Deserialize)]
struct Config {
    /// Substring-matched, forward-slash-normalized source paths where direct
    /// `sha2`/`sha1`/`md5` imports are temporarily permitted — intended for
    /// phased FIPS migration milestones. Empty by default (deny everywhere).
    #[serde(default)]
    hasher_allowed_paths: Vec<String>,
}

pub(crate) struct De0708NoNonFipsHasher {
    allowed_paths: Vec<String>,
}

impl De0708NoNonFipsHasher {
    pub fn new() -> Self {
        let config: Config = dylint_linting::config_or_default(crate::LIBRARY_NAME);
        Self {
            allowed_paths: config
                .hasher_allowed_paths
                .into_iter()
                .map(|p| p.trim().replace('\\', "/"))
                .filter(|p| !p.is_empty())
                .collect(),
        }
    }

    /// Returns true when `span`'s source file matches a configured allow-list
    /// entry, suppressing the lint there (a phased-FIPS exception).
    fn is_allowed(&self, cx: &EarlyContext<'_>, span: Span) -> bool {
        if self.allowed_paths.is_empty() {
            return false;
        }
        let Some(path) = filename_str(cx.sess().source_map(), span) else {
            return false;
        };
        let normalized = path.replace('\\', "/");
        self.allowed_paths
            .iter()
            .any(|allowed| normalized.contains(allowed.as_str()))
    }
}

dylint_linting::impl_early_lint! {
    /// ### What it does
    ///
    /// Prohibits imports of non-FIPS-validated hash crates (`sha2`, `sha1`, `md5`)
    /// outside a configurable allow-list of source paths.
    ///
    /// ### Why is this bad?
    ///
    /// These crates use pure-Rust RustCrypto implementations that are not
    /// FIPS-validated. While they may be present in the dependency graph via
    /// transitives, new *direct* usage should not be introduced without review.
    ///
    /// ### Configuration
    ///
    /// Exceptions are centralized in the consuming workspace's `dylint.toml`,
    /// which suits a phased FIPS rollout where paths are permitted now and
    /// removed as milestones land:
    ///
    /// ```toml
    /// [cargo-gears-lints]
    /// hasher_allowed_paths = ["libs/legacy-checksum/", "gears/foo/src/etag.rs"]
    /// ```
    ///
    /// Entries are substring-matched against forward-slash-normalized file
    /// paths. The list is empty by default (deny everywhere).
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad — direct sha2 import in application code
    /// use sha2::{Digest, Sha256};
    /// ```
    ///
    /// Use instead: route the operation through the validated crypto provider,
    /// or add the file to `hasher_allowed_paths` if the usage is reviewed and
    /// tracked against a FIPS milestone.
    #[doc = include_str!("../../docs/de07_security/de0708_no_non_fips_hasher/README.md")]
    pub DE0708_NO_NON_FIPS_HASHER,
    Deny,
    "non-FIPS-validated hasher import (sha2/sha1/md5) outside allow-list (DE0708)",
    De0708NoNonFipsHasher::new()
}

/// Crate names to detect (as they appear in `use` statements — hyphens become underscores).
const BANNED_CRATES: &[&str] = &["sha2", "sha1", "md5"];

/// Check if a resolved use-path matches one of the banned hasher crates.
fn is_banned_path(path: &str) -> bool {
    BANNED_CRATES
        .iter()
        .any(|crate_name| path == *crate_name || path.starts_with(&format!("{crate_name}::")))
}

/// Find the first banned path in a use tree (handles grouped imports).
fn find_banned_path(tree: &rustc_ast::UseTree) -> Option<String> {
    use_tree_to_strings(tree)
        .into_iter()
        .find(|path| is_banned_path(path))
}

impl EarlyLintPass for De0708NoNonFipsHasher {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Skip files in the configured allow-list (phased FIPS exceptions).
        if self.is_allowed(cx, item.span) {
            return;
        }

        let banned = match &item.kind {
            ItemKind::Use(use_tree) => find_banned_path(use_tree),
            ItemKind::ExternCrate(rename, ident) => {
                let name = match rename {
                    Some(sym) => sym.as_str(),
                    None => ident.name.as_str(),
                };
                is_banned_path(name).then(|| name.to_owned())
            }
            _ => None,
        };

        if let Some(path_str) = banned {
            let crate_root = path_str.split("::").next().unwrap_or(&path_str);
            cx.span_lint(DE0708_NO_NON_FIPS_HASHER, item.span, |diag| {
                diag.primary_message(format!(
                    "non-FIPS-validated hasher import detected: `{crate_root}` (DE0708)"
                ));
                diag.help(
                    "these crates use pure-Rust RustCrypto; allow-list the path via `hasher_allowed_paths` in dylint.toml if reviewed",
                );
                diag.note("see your project's FIPS dependency policy for details");
            });
        }
    }
}
