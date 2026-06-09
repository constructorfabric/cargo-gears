extern crate rustc_ast;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

dylint_linting::declare_pre_expansion_lint! {
    /// DE0204: DTOs Must Have ToSchema Derive
    ///
    /// All DTO types MUST derive `utoipa::ToSchema` for OpenAPI documentation.
    /// DTOs in api/rest need schema definitions for API documentation.
    ///
    /// ### Example: Bad
    ///
    /// ```rust,ignore
    /// // src/api/rest/dto.rs
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]  // ❌ Missing ToSchema
    /// pub struct UserDto {
    ///     pub id: String,
    /// }
    /// ```
    ///
    /// ### Example: Good
    ///
    /// ```rust,ignore
    /// // src/api/rest/dto.rs
    /// use serde::{Deserialize, Serialize};
    /// use utoipa::ToSchema;
    ///
    /// #[derive(Debug, Serialize, Deserialize, ToSchema)]  // ✅ Has ToSchema
    /// pub struct UserDto {
    ///     pub id: String,
    /// }
    /// ```
    #[doc = include_str!("../../docs/de02_api_layer/de0204_dtos_must_have_toschema_derive/README.md")]
    pub DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE,
    Deny,
    "DTO types must derive ToSchema for OpenAPI documentation (DE0204)"
}

impl EarlyLintPass for De0204DtosMustHaveToschemaDerive {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        check_dto_toschema_derive(cx, item);
    }
}

fn check_dto_toschema_derive(cx: &EarlyContext<'_>, item: &Item) {
    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    // Check if the type name ends with "Dto" suffix (case-insensitive)
    let item_name = match &item.kind {
        ItemKind::Struct(ident, _, _) => ident.name.as_str(),
        ItemKind::Enum(ident, _, _) => ident.name.as_str(),
        _ => return,
    };
    let item_name_lower = item_name.to_lowercase();
    if !item_name_lower.ends_with("dto") {
        return;
    }

    // Check for api_dto macro which adds ToSchema derive automatically
    if crate::lint_utils::has_api_dto_attribute(item) {
        return;
    }

    // Check for ToSchema derive
    let mut has_toschema = false;
    crate::lint_utils::check_derive_attrs(item, |meta_item, _attr| {
        let segments = crate::lint_utils::get_derive_path_segments(meta_item);
        // Check for ToSchema (bare or utoipa::ToSchema)
        if crate::lint_utils::is_utoipa_trait(&segments, "ToSchema") {
            has_toschema = true;
        }
    });

    // Report missing derive
    if !has_toschema {
        cx.span_lint(DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE, item.span, |diag| {
            diag.primary_message("api/rest type is missing required ToSchema derive (DE0204)");
            diag.help("DTOs in api/rest must derive ToSchema for OpenAPI documentation");
        });
    }
}
