extern crate rustc_ast;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use crate::lint_utils::is_in_domain_path;

dylint_linting::declare_pre_expansion_lint! {
    /// ### What it does
    ///
    /// Checks that structs and enums in domain modules do not derive ToSchema.
    ///
    /// ### Why is this bad?
    ///
    /// Domain models should remain independent of OpenAPI documentation concerns.
    /// ToSchema is for API documentation and should only be used on DTOs in the API layer.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Bad - domain model derives ToSchema
    /// mod domain {
    ///     use utoipa::ToSchema;
    ///     #[derive(ToSchema)]
    ///     pub struct Product { pub id: String }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Good - domain model without ToSchema
    /// mod domain {
    ///     pub struct Product { pub id: String }
    /// }
    ///
    /// // Separate DTO in API layer
    /// mod api {
    ///     use utoipa::ToSchema;
    ///     use serde::{Serialize, Deserialize};
    ///     #[derive(Serialize, Deserialize, ToSchema)]
    ///     pub struct ProductDto { pub id: String }
    /// }
    /// ```
    #[doc = include_str!("../../docs/de01_domain_layer/de0102_no_toschema_in_domain/README.md")]
    pub DE0102_NO_TOSCHEMA_IN_CONTRACT,
    Deny,
    "domain models should not have ToSchema derive (DE0102)"
}

impl EarlyLintPass for De0102NoToschemaInContract {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Only check structs and enums
        if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
            return;
        }

        if !is_in_domain_path(cx.sess().source_map(), item.span) {
            return;
        }

        // Check for ToSchema derives
        crate::lint_utils::check_derive_attrs(item, |meta_item, attr| {
            let segments = crate::lint_utils::get_derive_path_segments(meta_item);

            // Check if this is a utoipa ToSchema
            // Handles: ToSchema, utoipa::ToSchema, ::utoipa::ToSchema
            if crate::lint_utils::is_utoipa_trait(&segments, "ToSchema") {
                cx.span_lint(DE0102_NO_TOSCHEMA_IN_CONTRACT, attr.span, |diag| {
                    diag.primary_message("domain type should not derive `ToSchema` (DE0102)");
                    diag.help("ToSchema is an OpenAPI concern; use DTOs in api/rest/ instead");
                });
            }
        });
    }
}
