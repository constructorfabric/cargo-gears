extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{
    AssocItemKind, Attribute, FieldDef, Item, ItemKind, Visibility, VisibilityKind, visit,
    visit::Visitor,
};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::{Span, symbol::kw, symbol::sym};
use std::{collections::HashSet, path::Path};

#[derive(Default, serde::Deserialize)]
struct Config {
    #[serde(default)]
    de1202_excluded_crates: Vec<String>,
}

pub(crate) struct De1202MissingDocsForPubCrate {
    excluded_crates: HashSet<String>,
}

impl De1202MissingDocsForPubCrate {
    pub fn new() -> Self {
        let config: Config = dylint_linting::config_or_default(crate::LIBRARY_NAME);
        Self {
            excluded_crates: normalize_excluded_crates(config.de1202_excluded_crates),
        }
    }

    fn crate_is_excluded(&self, cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) -> bool {
        let rustc_crate_name = cx
            .sess()
            .opts
            .crate_name
            .clone()
            .or_else(|| source_crate_name(cx, krate));
        let rustc_crate_is_excluded = rustc_crate_name
            .as_deref()
            .is_some_and(|crate_name| is_name_excluded(&self.excluded_crates, crate_name));
        let cargo_package_is_excluded = std::env::var("CARGO_PKG_NAME")
            .ok()
            .is_some_and(|package_name| is_name_excluded(&self.excluded_crates, &package_name));

        rustc_crate_is_excluded || cargo_package_is_excluded
    }
}

dylint_linting::impl_pre_expansion_lint! {
    /// DE1202: Crate-public APIs must have documentation
    ///
    /// Complements rustc's `missing_docs` lint by requiring documentation for
    /// APIs declared with `pub(crate)`. The built-in lint covers externally
    /// exported `pub` APIs but intentionally ignores crate-public APIs.
    /// Existing crates can be temporarily excluded with
    /// `de1202_excluded_crates` in `dylint.toml`.
    #[doc = include_str!("de1202_missing_docs_for_pub_crate/README.md")]
    pub DE1202_MISSING_DOCS_FOR_PUB_CRATE,
    Deny,
    "pub(crate) APIs must have documentation (DE1202)",
    De1202MissingDocsForPubCrate::new()
}

impl EarlyLintPass for De1202MissingDocsForPubCrate {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) {
        if self.crate_is_excluded(cx, krate) {
            return;
        }

        MissingDocsVisitor {
            cx,
            in_test_scope: false,
            in_crate_public_module: false,
        }
        .visit_crate(krate);
    }
}

struct MissingDocsVisitor<'a, 'cx> {
    cx: &'a EarlyContext<'cx>,
    in_test_scope: bool,
    in_crate_public_module: bool,
}

impl<'ast> Visitor<'ast> for MissingDocsVisitor<'_, '_> {
    fn visit_item(&mut self, item: &'ast Item) {
        let previous_test_scope = self.in_test_scope;
        let previous_crate_public_module = self.in_crate_public_module;
        self.in_test_scope =
            previous_test_scope || is_test_item(&item.attrs) || is_test_path(self.cx, item.span);

        let item_is_crate_public = is_pub_crate(&item.vis)
            || (previous_crate_public_module && matches!(item.vis.kind, VisibilityKind::Public));
        if !self.in_test_scope && !item.span.from_expansion() {
            self.check_item(item, item_is_crate_public);
        }

        if matches!(item.kind, ItemKind::Mod(..)) {
            self.in_crate_public_module = item_is_crate_public;
        }
        visit::walk_item(self, item);
        self.in_crate_public_module = previous_crate_public_module;
        self.in_test_scope = previous_test_scope;
    }
}

impl MissingDocsVisitor<'_, '_> {
    fn check_item(&self, item: &Item, item_is_crate_public: bool) {
        if item_is_crate_public && item_kind_requires_docs(&item.kind) {
            check_docs(self.cx, &item.attrs, item.span, item_kind_name(&item.kind));
        }

        if is_doc_hidden(&item.attrs) {
            return;
        }

        match &item.kind {
            ItemKind::Struct(_, _, data) | ItemKind::Union(_, _, data) => {
                for field in data.fields() {
                    if is_pub_crate(&field.vis)
                        || (item_is_crate_public
                            && matches!(field.vis.kind, VisibilityKind::Public))
                    {
                        check_field_docs(self.cx, field);
                    }
                }
            }
            ItemKind::Enum(_, _, definition) if item_is_crate_public => {
                for variant in &definition.variants {
                    check_docs(self.cx, &variant.attrs, variant.span, "enum variant");

                    for field in variant.data.fields() {
                        if field.ident.is_some() {
                            check_field_docs(self.cx, field);
                        }
                    }
                }
            }
            ItemKind::Trait(trait_definition) if item_is_crate_public => {
                for associated_item in &trait_definition.items {
                    if associated_item_kind_requires_docs(&associated_item.kind) {
                        check_docs(
                            self.cx,
                            &associated_item.attrs,
                            associated_item.span,
                            "trait item",
                        );
                    }
                }
            }
            _ => {}
        }

        self.check_crate_public_impl_items(item);
    }

    fn check_crate_public_impl_items(&self, item: &Item) {
        let ItemKind::Impl(implementation) = &item.kind else {
            return;
        };

        // Trait implementations inherit documentation from the trait declaration.
        if implementation.of_trait.is_some() {
            return;
        }

        for associated_item in &implementation.items {
            if is_pub_crate(&associated_item.vis)
                || (self.in_crate_public_module
                    && matches!(associated_item.vis.kind, VisibilityKind::Public))
            {
                check_docs(
                    self.cx,
                    &associated_item.attrs,
                    associated_item.span,
                    "associated item",
                );
            }
        }
    }
}

fn associated_item_kind_requires_docs(kind: &AssocItemKind) -> bool {
    matches!(
        kind,
        AssocItemKind::Const(..) | AssocItemKind::Fn(..) | AssocItemKind::Type(..)
    )
}

fn item_kind_requires_docs(kind: &ItemKind) -> bool {
    matches!(
        kind,
        ItemKind::Const(..)
            | ItemKind::Enum(..)
            | ItemKind::Fn(..)
            | ItemKind::MacroDef(..)
            | ItemKind::Mod(..)
            | ItemKind::Static(..)
            | ItemKind::Struct(..)
            | ItemKind::Trait(..)
            | ItemKind::TraitAlias(..)
            | ItemKind::TyAlias(..)
            | ItemKind::Union(..)
    )
}

fn item_kind_name(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Const(..) => "constant",
        ItemKind::Enum(..) => "enum",
        ItemKind::Fn(..) => "function",
        ItemKind::MacroDef(..) => "macro",
        ItemKind::Mod(..) => "module",
        ItemKind::Static(..) => "static",
        ItemKind::Struct(..) => "struct",
        ItemKind::Trait(..) => "trait",
        ItemKind::TraitAlias(..) => "trait alias",
        ItemKind::TyAlias(..) => "type alias",
        ItemKind::Union(..) => "union",
        _ => "item",
    }
}

fn source_crate_name(cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) -> Option<String> {
    let first_item = krate.items.first()?;
    let filename = crate::lint_utils::filename_str(cx.sess().source_map(), first_item.span)?;
    Path::new(&filename)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
}

fn normalize_crate_name(crate_name: &str) -> String {
    crate_name.trim().replace('-', "_")
}

fn normalize_excluded_crates(crate_names: Vec<String>) -> HashSet<String> {
    crate_names
        .into_iter()
        .map(|crate_name| normalize_crate_name(&crate_name))
        .filter(|crate_name| !crate_name.is_empty())
        .collect()
}

fn is_name_excluded(excluded_crates: &HashSet<String>, crate_name: &str) -> bool {
    excluded_crates.contains(&normalize_crate_name(crate_name))
}

fn is_pub_crate(visibility: &Visibility) -> bool {
    let VisibilityKind::Restricted { path, .. } = &visibility.kind else {
        return false;
    };

    path.segments.len() == 1 && path.segments[0].ident.name == kw::Crate
}

fn check_field_docs(cx: &EarlyContext<'_>, field: &FieldDef) {
    check_docs(cx, &field.attrs, field.span, "field");
}

fn check_docs(cx: &EarlyContext<'_>, attrs: &[Attribute], span: Span, kind: &str) {
    if has_nonempty_docs(attrs) || is_doc_hidden(attrs) {
        return;
    }

    cx.span_lint(DE1202_MISSING_DOCS_FOR_PUB_CRATE, span, |diag| {
        diag.primary_message(format!(
            "crate-public {kind} is missing documentation (DE1202)"
        ));
        diag.help("add a non-empty `///` doc comment or `#[doc = ...]` attribute");
    });
}

fn has_nonempty_docs(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if let Some(doc) = attr.doc_str() {
            return !doc.as_str().trim().is_empty();
        }

        // At pre-expansion time, macro-backed documentation such as
        // `#[doc = include_str!("README.md")]` is not yet a string literal.
        attr.has_name(sym::doc) && attr.meta_item_list().is_none()
    })
}

fn is_doc_hidden(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.has_name(sym::doc)
            && attr
                .meta_item_list()
                .is_some_and(|items| items.iter().any(|item| item.has_name(sym::hidden)))
    })
}

fn is_test_item(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.has_name(sym::test) {
            return true;
        }

        attr.has_name(sym::cfg)
            && attr
                .meta_item_list()
                .is_some_and(|items| items.iter().any(|item| item.has_name(sym::test)))
    })
}

fn is_test_path(cx: &EarlyContext<'_>, span: Span) -> bool {
    let Some(path) = crate::lint_utils::filename_str(cx.sess().source_map(), span) else {
        return false;
    };
    let effective_path = simulated_path(&path).unwrap_or(path).replace('\\', "/");

    effective_path.contains("/tests/")
        || effective_path.starts_with("tests/")
        || effective_path.ends_with("_tests.rs")
}

fn simulated_path(path: &str) -> Option<String> {
    if !crate::lint_utils::is_temp_path(path) {
        return None;
    }

    let source = std::fs::read_to_string(path).ok()?;
    source.lines().next().and_then(|line| {
        line.trim()
            .strip_prefix("// simulated_dir=")
            .map(ToOwned::to_owned)
    })
}

#[cfg(test)]
mod tests {
    use super::{Config, is_name_excluded, normalize_excluded_crates};

    #[test]
    fn excluded_crate_matches_exact_name() {
        let excluded = normalize_excluded_crates(vec!["legacy_crate".to_string()]);
        assert!(is_name_excluded(&excluded, "legacy_crate"));
        assert!(!is_name_excluded(&excluded, "new_crate"));
    }

    #[test]
    fn hyphenated_package_name_matches_rustc_crate_name() {
        let excluded = normalize_excluded_crates(vec!["legacy-crate".to_string()]);
        assert!(is_name_excluded(&excluded, "legacy-crate"));
        assert!(is_name_excluded(&excluded, "legacy_crate"));
    }

    #[test]
    fn config_deserializes_excluded_crates() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "de1202_excluded_crates": ["legacy-crate"]
        }))
        .expect("DE1202 config should deserialize");
        let excluded = normalize_excluded_crates(config.de1202_excluded_crates);
        assert!(is_name_excluded(&excluded, "legacy_crate"));
    }

    #[test]
    fn empty_exclusion_configuration_excludes_nothing() {
        let config = Config::default();
        let excluded = normalize_excluded_crates(config.de1202_excluded_crates);
        assert!(!is_name_excluded(&excluded, "legacy_crate"));
    }

    #[test]
    fn empty_and_duplicate_entries_are_normalized() {
        let excluded = normalize_excluded_crates(vec![
            "".to_string(),
            " legacy-crate ".to_string(),
            "legacy_crate".to_string(),
        ]);
        assert_eq!(excluded.len(), 1);
        assert!(is_name_excluded(&excluded, "legacy_crate"));
    }
}
