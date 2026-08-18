extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::{is_doc_hidden, is_from_proc_macro};
use rustc_hir::def_id::{LOCAL_CRATE, LocalDefId};
use rustc_hir::{
    Attribute, Body, BodyId, FieldDef, ForeignItem, HirId, ImplItem, Item, ItemKind, Node,
    TraitItem, Variant,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::middle::privacy::Level;
use rustc_middle::ty::Visibility;
use rustc_span::Span;
use std::collections::HashSet;

#[derive(Default, serde::Deserialize)]
struct Config {
    #[serde(default)]
    de1202_excluded_crates: Vec<String>,
}

pub(crate) struct De1202MissingDocsForPubCrate {
    excluded_crates: HashSet<String>,
    skip_crate: bool,
    attr_depth: u32,
    doc_hidden_depth: u32,
    automatically_derived_depth: u32,
    in_body: Option<BodyId>,
    crate_public_modules: Vec<bool>,
    item_requires_docs: Vec<bool>,
}

impl De1202MissingDocsForPubCrate {
    pub fn new() -> Self {
        let config: Config = dylint_linting::config_or_default(crate::LIBRARY_NAME);
        Self {
            excluded_crates: normalize_excluded_crates(config.de1202_excluded_crates),
            skip_crate: false,
            attr_depth: 0,
            doc_hidden_depth: 0,
            automatically_derived_depth: 0,
            in_body: None,
            crate_public_modules: Vec::new(),
            item_requires_docs: Vec::new(),
        }
    }

    fn crate_is_excluded(&self, cx: &LateContext<'_>) -> bool {
        let rustc_crate_name = cx.tcx.crate_name(LOCAL_CRATE);
        let rustc_crate_is_excluded =
            is_name_excluded(&self.excluded_crates, rustc_crate_name.as_str());
        let cargo_package_is_excluded = std::env::var("CARGO_PKG_NAME")
            .ok()
            .is_some_and(|package_name| is_name_excluded(&self.excluded_crates, &package_name));

        rustc_crate_is_excluded || cargo_package_is_excluded
    }

    fn is_in_exempt_scope(&self) -> bool {
        self.skip_crate
            || self.doc_hidden_depth != 0
            || self.automatically_derived_depth != 0
            || self.in_body.is_some()
    }

    fn check_definition(
        &self,
        cx: &LateContext<'_>,
        hir_id: HirId,
        span: Span,
        kind: &str,
        requires_docs: bool,
    ) {
        if self.is_in_exempt_scope()
            || span.from_expansion()
            || is_test_path(cx, span)
            || !requires_docs
            || has_nonempty_docs(cx.tcx.hir_attrs(hir_id))
        {
            return;
        }

        cx.span_lint(DE1202_MISSING_DOCS_FOR_PUB_CRATE, span, |diag| {
            diag.primary_message(format!(
                "crate-public {kind} is missing documentation (DE1202)"
            ));
            diag.help("add a non-empty `///` doc comment or `#[doc = ...]` attribute");
        });
    }
}

dylint_linting::impl_late_lint! {
    /// DE1202: Crate-public APIs must have documentation
    ///
    /// Complements rustc's `missing_docs` lint by requiring documentation for
    /// APIs declared with `pub(crate)` or whose effective visibility is limited
    /// to the crate. The built-in lint covers externally exported APIs.
    /// Existing crates can be temporarily excluded with
    /// `de1202_excluded_crates` in `dylint.toml`.
    #[doc = include_str!("de1202_missing_docs_for_pub_crate/README.md")]
    pub DE1202_MISSING_DOCS_FOR_PUB_CRATE,
    Deny,
    "crate-public APIs must have documentation (DE1202)",
    De1202MissingDocsForPubCrate::new()
}

impl<'tcx> LateLintPass<'tcx> for De1202MissingDocsForPubCrate {
    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        self.skip_crate = cx.tcx.sess.opts.test || self.crate_is_excluded(cx);
    }

    fn check_attributes(&mut self, _: &LateContext<'tcx>, attrs: &'tcx [Attribute]) {
        self.attr_depth += 1;
        if self.doc_hidden_depth == 0 && is_doc_hidden(attrs) {
            self.doc_hidden_depth = self.attr_depth;
        }
    }

    fn check_attributes_post(&mut self, _: &LateContext<'tcx>, _: &'tcx [Attribute]) {
        self.attr_depth -= 1;
        if self.attr_depth < self.doc_hidden_depth {
            self.doc_hidden_depth = 0;
        }
        if self.attr_depth < self.automatically_derived_depth {
            self.automatically_derived_depth = 0;
        }
    }

    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let public_is_crate_public = self.crate_public_modules.last().copied().unwrap_or(false);
        let requires_docs = declaration_requires_docs(cx, item.vis_span, public_is_crate_public);

        let kind = match item.kind {
            ItemKind::Impl(..) => {
                if cx
                    .tcx
                    .is_automatically_derived(item.owner_id.def_id.to_def_id())
                {
                    self.automatically_derived_depth = self.attr_depth;
                }
                None
            }
            ItemKind::Const(..) => Some("constant"),
            ItemKind::Enum(..) => Some("enum"),
            ItemKind::Fn { .. } => Some("function"),
            ItemKind::Macro(..) => Some("macro"),
            ItemKind::Mod(..) => Some("module"),
            ItemKind::Static(..) => Some("static"),
            ItemKind::Struct(..) => Some("struct"),
            ItemKind::Trait(..) => Some("trait"),
            ItemKind::TraitAlias(..) => Some("trait alias"),
            ItemKind::TyAlias(..) => Some("type alias"),
            ItemKind::Union(..) => Some("union"),
            ItemKind::ExternCrate(..)
            | ItemKind::ForeignMod { .. }
            | ItemKind::GlobalAsm { .. }
            | ItemKind::Use(..) => None,
        };

        if let Some(kind) = kind
            && !is_from_proc_macro(cx, item)
        {
            self.check_definition(cx, item.hir_id(), item.span, kind, requires_docs);
        }

        self.item_requires_docs.push(requires_docs);
        if matches!(item.kind, ItemKind::Mod(..)) {
            self.crate_public_modules.push(requires_docs);
        }
    }

    fn check_item_post(&mut self, _: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        self.item_requires_docs
            .pop()
            .expect("DE1202 item visibility stack should be balanced");
        if matches!(item.kind, ItemKind::Mod(..)) {
            self.crate_public_modules
                .pop()
                .expect("DE1202 module visibility stack should be balanced");
        }
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx TraitItem<'tcx>) {
        if !is_from_proc_macro(cx, item) {
            self.check_definition(
                cx,
                item.hir_id(),
                item.span,
                "trait item",
                self.item_requires_docs.last().copied().unwrap_or(false),
            );
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx ImplItem<'tcx>) {
        let is_inherent_item = matches!(
            cx.tcx.parent_hir_node(item.hir_id()),
            Node::Item(parent)
                if matches!(parent.kind, ItemKind::Impl(implementation) if implementation.of_trait.is_none())
        );

        if is_inherent_item && !is_from_proc_macro(cx, item) {
            self.check_definition(
                cx,
                item.hir_id(),
                item.span,
                "associated item",
                item.vis_span().is_some_and(|visibility_span| {
                    match declared_visibility(cx, visibility_span) {
                        DeclaredVisibility::Crate => true,
                        DeclaredVisibility::Public => {
                            is_effectively_crate_public(cx, item.owner_id.def_id)
                        }
                        DeclaredVisibility::Inherited | DeclaredVisibility::OtherRestricted => {
                            false
                        }
                    }
                }),
            );
        }
    }

    fn check_field_def(&mut self, cx: &LateContext<'tcx>, field: &'tcx FieldDef<'tcx>) {
        let is_positional_enum_field = field.is_positional()
            && matches!(cx.tcx.parent_hir_node(field.hir_id), Node::Variant(..));
        if !is_positional_enum_field && !is_from_proc_macro(cx, field) {
            let parent_requires_docs = self.item_requires_docs.last().copied().unwrap_or(false);
            let requires_docs = if matches!(cx.tcx.parent_hir_node(field.hir_id), Node::Variant(..))
            {
                parent_requires_docs
            } else {
                declaration_requires_docs(cx, field.vis_span, parent_requires_docs)
            };
            self.check_definition(cx, field.hir_id, field.span, "field", requires_docs);
        }
    }

    fn check_variant(&mut self, cx: &LateContext<'tcx>, variant: &'tcx Variant<'tcx>) {
        if !is_from_proc_macro(cx, variant) {
            self.check_definition(
                cx,
                variant.hir_id,
                variant.span,
                "enum variant",
                self.item_requires_docs.last().copied().unwrap_or(false),
            );
        }
    }

    fn check_foreign_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx ForeignItem<'tcx>) {
        self.check_definition(
            cx,
            item.hir_id(),
            item.span,
            "foreign item",
            declaration_requires_docs(
                cx,
                item.vis_span,
                self.crate_public_modules.last().copied().unwrap_or(false),
            ),
        );
    }

    fn check_body(&mut self, _: &LateContext<'tcx>, body: &Body<'tcx>) {
        if self.doc_hidden_depth == 0
            && self.automatically_derived_depth == 0
            && self.in_body.is_none()
        {
            self.in_body = Some(body.id());
        }
    }

    fn check_body_post(&mut self, _: &LateContext<'tcx>, body: &Body<'tcx>) {
        if self.in_body == Some(body.id()) {
            self.in_body = None;
        }
    }
}

fn declaration_requires_docs(
    cx: &LateContext<'_>,
    visibility_span: Span,
    public_is_crate_public: bool,
) -> bool {
    match declared_visibility(cx, visibility_span) {
        DeclaredVisibility::Crate => true,
        DeclaredVisibility::Public => public_is_crate_public,
        DeclaredVisibility::Inherited | DeclaredVisibility::OtherRestricted => false,
    }
}

fn is_effectively_crate_public(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    let Some(effective_visibility) = cx.effective_visibilities.effective_vis(def_id) else {
        return false;
    };

    if effective_visibility.at_level(Level::Reexported).is_public() {
        return false;
    }

    let visibility = effective_visibility.at_level(Level::Reachable);
    visibility.is_public()
        || matches!(
            visibility,
            Visibility::Restricted(module) if module.is_top_level_module()
        )
}

#[derive(Clone, Copy)]
enum DeclaredVisibility {
    Inherited,
    Public,
    Crate,
    OtherRestricted,
}

fn declared_visibility(cx: &LateContext<'_>, span: Span) -> DeclaredVisibility {
    let Ok(source) = cx.sess().source_map().span_to_snippet(span) else {
        return DeclaredVisibility::Inherited;
    };
    let compact = compact_visibility_source(&source);

    match compact.as_str() {
        "pub" => DeclaredVisibility::Public,
        "pub(crate)" | "pub(incrate)" => DeclaredVisibility::Crate,
        "" => DeclaredVisibility::Inherited,
        _ => DeclaredVisibility::OtherRestricted,
    }
}

fn compact_visibility_source(source: &str) -> String {
    let mut result = String::new();
    let mut characters = source.chars().peekable();
    let mut block_comment_depth = 0_u32;

    while let Some(character) = characters.next() {
        if block_comment_depth != 0 {
            if character == '/' && characters.peek() == Some(&'*') {
                characters.next();
                block_comment_depth += 1;
            } else if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                block_comment_depth -= 1;
            }
            continue;
        }

        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            block_comment_depth = 1;
        } else if character == '/' && characters.peek() == Some(&'/') {
            break;
        } else if !character.is_whitespace() {
            result.push(character);
        }
    }

    result
}

fn has_nonempty_docs(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.doc_str()
            .is_some_and(|docs| !docs.as_str().trim().is_empty())
    })
}

fn is_test_path(cx: &LateContext<'_>, span: Span) -> bool {
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

#[cfg(test)]
mod tests {
    use super::{Config, compact_visibility_source, is_name_excluded, normalize_excluded_crates};

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
    fn visibility_source_normalization_ignores_whitespace_and_comments() {
        assert_eq!(
            compact_visibility_source("pub ( in crate )"),
            "pub(incrate)"
        );
        assert_eq!(
            compact_visibility_source("pub(in /* nested /* comment */ */ crate)"),
            "pub(incrate)"
        );
        assert_eq!(compact_visibility_source("pub(super)"), "pub(super)");
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
