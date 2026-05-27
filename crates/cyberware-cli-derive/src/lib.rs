use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, Meta, parse_macro_input};

/// Derive macro that harvests `#[doc = "..."]` comments and `#[serde(...)]`
/// attributes from a struct or enum to implement the `HelpSchema` trait.
///
/// The generated implementation provides:
/// - `help_name()` → type name
/// - `help_doc()` → concatenated struct/enum-level doc comments
/// - `help_fields()` → `Vec<FieldHelp>` with per-field metadata
#[proc_macro_derive(HelpSchema, attributes(help))]
pub fn derive_help_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let type_doc = extract_doc_comments(&input.attrs);
    let krate = extract_crate_path(&input.attrs);

    let field_exprs = match &input.data {
        Data::Struct(data) => fields_to_exprs(&data.fields, &krate),
        Data::Enum(data) => enum_variants_to_exprs(data, &krate),
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "HelpSchema cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #krate::help::HelpSchema for #name {
            fn help_name() -> &'static str {
                stringify!(#name)
            }

            fn help_doc() -> &'static str {
                #type_doc
            }

            fn help_fields() -> ::std::vec::Vec<#krate::help::FieldHelp> {
                ::std::vec![#(#field_exprs),*]
            }
        }
    };

    expanded.into()
}

/// Extract `#[help(crate_path = "...")]` or default to `crate`.
#[allow(clippy::expect_used)]
fn extract_crate_path(attrs: &[syn::Attribute]) -> syn::Path {
    for attr in attrs {
        if !attr.path().is_ident("help") {
            continue;
        }
        if let Ok(nested) = attr
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        {
            for meta in &nested {
                if let Meta::NameValue(nv) = meta
                    && nv.path.is_ident("crate_path")
                    && let syn::Expr::Lit(expr_lit) = &nv.value
                    && let Lit::Str(lit) = &expr_lit.lit
                {
                    return lit.parse::<syn::Path>().unwrap_or_else(|_| {
                        syn::parse_str("crate").expect("crate is a valid path")
                    });
                }
            }
        }
    }
    syn::parse_str("crate").expect("crate is a valid path")
}

/// Collect all `#[doc = "..."]` attributes and join them into a single string
/// literal.
fn extract_doc_comments(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let mut lines: Vec<String> = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &nv.value
            && let Lit::Str(lit) = &expr_lit.lit
        {
            lines.push(lit.value());
        }
    }
    let joined = lines.join("\n").trim().to_owned();
    quote! { #joined }
}

/// Extract serde `rename` value from an attribute list.
fn extract_serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        if let Ok(nested) = attr
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        {
            for meta in &nested {
                if let Meta::NameValue(nv) = meta
                    && nv.path.is_ident("rename")
                    && let syn::Expr::Lit(expr_lit) = &nv.value
                    && let Lit::Str(lit) = &expr_lit.lit
                {
                    return Some(lit.value());
                }
            }
        }
    }
    None
}

/// Check if `serde(default)` or `serde(default = "...")` is present.
fn extract_serde_default(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        if let Ok(nested) = attr
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        {
            for meta in &nested {
                match meta {
                    Meta::Path(p) if p.is_ident("default") => {
                        return Some("true".to_owned());
                    }
                    Meta::NameValue(nv)
                        if nv.path.is_ident("default")
                            && let syn::Expr::Lit(expr_lit) = &nv.value
                            && let Lit::Str(lit) = &expr_lit.lit =>
                    {
                        return Some(lit.value());
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

/// Check if a type path looks like `Option<T>`.
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        return seg.ident == "Option";
    }
    false
}

/// Produce a short human-readable representation of a type.
fn type_to_string(ty: &syn::Type) -> String {
    use quote::ToTokens;
    let ts = ty.to_token_stream();
    let raw = ts.to_string();
    // Clean up spacing artifacts from token stream
    raw.replace(" < ", "<")
        .replace(" > ", ">")
        .replace(" >", ">")
        .replace("< ", "<")
        .replace(" , ", ", ")
}

/// Build `FieldHelp` expressions for struct fields.
fn fields_to_exprs(fields: &Fields, krate: &syn::Path) -> Vec<proc_macro2::TokenStream> {
    let mut exprs = Vec::new();

    let named = match fields {
        Fields::Named(named) => &named.named,
        _ => return exprs,
    };

    for field in named {
        let field_name = field
            .ident
            .as_ref()
            .map_or_else(String::new, ToString::to_string);

        let display_name = extract_serde_rename(&field.attrs).unwrap_or_else(|| field_name.clone());

        let field_doc = extract_doc_comments(&field.attrs);
        let field_type_str = type_to_string(&field.ty);
        let optional = is_option_type(&field.ty);
        let has_default = extract_serde_default(&field.attrs).is_some();

        exprs.push(quote! {
            #krate::help::FieldHelp {
                name: #display_name,
                field_type: #field_type_str,
                doc: #field_doc,
                optional: #optional,
                has_default: #has_default,
            }
        });
    }

    exprs
}

/// Build `FieldHelp` expressions for enum variants (used for tagged enums).
fn enum_variants_to_exprs(
    data: &syn::DataEnum,
    krate: &syn::Path,
) -> Vec<proc_macro2::TokenStream> {
    let mut exprs = Vec::new();

    for variant in &data.variants {
        let variant_name = variant.ident.to_string();
        let display_name =
            extract_serde_rename(&variant.attrs).unwrap_or_else(|| variant_name.clone());
        let variant_doc = extract_doc_comments(&variant.attrs);

        let field_type_str = match &variant.fields {
            Fields::Named(f) => {
                let field_strs: Vec<String> = f
                    .named
                    .iter()
                    .map(|f| {
                        let name = f
                            .ident
                            .as_ref()
                            .map_or_else(|| "_".to_owned(), ToString::to_string);
                        let ty = type_to_string(&f.ty);
                        format!("{name}: {ty}")
                    })
                    .collect();
                format!("{{ {} }}", field_strs.join(", "))
            }
            Fields::Unnamed(f) => {
                let types: Vec<String> = f.unnamed.iter().map(|f| type_to_string(&f.ty)).collect();
                format!("({})", types.join(", "))
            }
            Fields::Unit => String::new(),
        };

        exprs.push(quote! {
            #krate::help::FieldHelp {
                name: #display_name,
                field_type: #field_type_str,
                doc: #variant_doc,
                optional: false,
                has_default: false,
            }
        });
    }

    exprs
}
