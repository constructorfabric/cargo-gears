use anyhow::{Context, bail};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Attribute, File, Item, ItemMod, Meta, UseTree};

/// Returned by [`resolve_rust_path`] when the requested path does not exist.
///
/// This sentinel lets callers distinguish a "not found" outcome from real
/// failures (I/O errors, parse errors, etc.) via [`anyhow::Error::is`].
#[derive(Debug)]
pub struct NotFoundError(pub String);

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NotFoundError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRustPath {
    pub source_path: PathBuf,
    pub source: String,
}

pub fn resolve_rust_path(
    root_source_path: &Path,
    segments: &[&str],
) -> anyhow::Result<ResolvedRustPath> {
    resolve_in_file(root_source_path, segments)
}

pub fn extract_reexport_target(
    source: &str,
    matched_name: &str,
) -> anyhow::Result<Option<Vec<String>>> {
    let file = syn::parse_file(source)
        .context("failed to parse resolved source for re-export analysis")?;
    let [item] = file.items.as_slice() else {
        return Ok(None);
    };
    let Item::Use(item_use) = item else {
        return Ok(None);
    };

    Ok(find_use_target(&item_use.tree, matched_name, &[]))
}

fn resolve_in_file(file_path: &Path, segments: &[&str]) -> anyhow::Result<ResolvedRustPath> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read Rust source from {}", file_path.display()))?;
    let mut parsed = syn::parse_file(&content)
        .with_context(|| format!("failed to parse Rust source from {}", file_path.display()))?;

    if segments.is_empty() {
        parsed.items = parsed.items.into_iter().filter_map(filter_item).collect();
        return Ok(ResolvedRustPath {
            source_path: file_path.to_path_buf(),
            source: render_file(&parsed),
        });
    }

    resolve_in_items(file_path, parsed.items, segments)
}

fn resolve_in_items(
    current_file: &Path,
    items: Vec<Item>,
    segments: &[&str],
) -> anyhow::Result<ResolvedRustPath> {
    let segment = segments[0];
    let mut use_fallback = None;

    for item in items {
        if is_test_item(&item) {
            continue;
        }

        if item_matches_name(&item, segment) {
            return if let Item::Mod(module) = item {
                resolve_module(current_file, module, &segments[1..])
            } else if segments.len() == 1 {
                Ok(ResolvedRustPath {
                    source_path: current_file.to_path_buf(),
                    source: render_item(
                        filter_item(item).context("resolved item was filtered out unexpectedly")?,
                    ),
                })
            } else {
                bail!(
                    "path continues past non-module item '{}' in {}",
                    segment,
                    current_file.display()
                );
            };
        }

        // Re-exports (`use` items) are captured only as a fallback rather than
        // matched immediately.  This avoids shadowing direct definitions and
        // ensures that a direct match earlier in the item list is always
        // preferred over a re-export that happens to expose the same name.
        if segments.len() == 1
            && let Item::Use(use_item) = item
            && use_tree_contains_name(&use_item.tree, segment)
        {
            use_fallback = Some(ResolvedRustPath {
                source_path: current_file.to_path_buf(),
                source: render_item(Item::Use(use_item)),
            });
        }
    }

    if let Some(use_fallback) = use_fallback {
        return Ok(use_fallback);
    }

    Err(anyhow::Error::new(NotFoundError(format!(
        "could not find '{}' in {}",
        segment,
        current_file.display()
    ))))
}

fn resolve_module(
    current_file: &Path,
    mut module: ItemMod,
    remaining_segments: &[&str],
) -> anyhow::Result<ResolvedRustPath> {
    if remaining_segments.is_empty() {
        if module.content.is_some() {
            return Ok(ResolvedRustPath {
                source_path: current_file.to_path_buf(),
                source: render_item(Item::Mod(filter_inline_module(module))),
            });
        }

        let module_file = resolve_module_file(current_file, &module)?;
        return resolve_in_file(&module_file, &[]);
    }

    if let Some((_, items)) = module.content.take() {
        return resolve_in_items(current_file, items, remaining_segments);
    }

    let module_file = resolve_module_file(current_file, &module)?;
    resolve_in_file(&module_file, remaining_segments)
}

fn resolve_module_file(current_file: &Path, module: &ItemMod) -> anyhow::Result<PathBuf> {
    if let Some(path) = module_path_override(module) {
        let base_dir = current_file.parent().with_context(|| {
            format!(
                "no parent directory available for {}",
                current_file.display()
            )
        })?;
        let overridden_path = base_dir.join(path);
        if overridden_path.is_file() {
            return Ok(overridden_path);
        }

        bail!(
            "module path override '{}' does not exist for {}",
            overridden_path.display(),
            module.ident
        );
    }

    let module_dir = module_search_dir(current_file)?;
    let file_candidate = module_dir.join(format!("{}.rs", module.ident));
    if file_candidate.is_file() {
        return Ok(file_candidate);
    }

    let mod_candidate = module_dir.join(module.ident.to_string()).join("mod.rs");
    if mod_candidate.is_file() {
        return Ok(mod_candidate);
    }

    bail!(
        "could not find source file for module '{}' referenced from {}",
        module.ident,
        current_file.display()
    )
}

fn module_search_dir(current_file: &Path) -> anyhow::Result<PathBuf> {
    let parent = current_file.parent().with_context(|| {
        format!(
            "no parent directory available for {}",
            current_file.display()
        )
    })?;
    let stem = current_file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .with_context(|| format!("invalid file stem for {}", current_file.display()))?;

    Ok(match stem {
        "lib" | "main" | "mod" => parent.to_path_buf(),
        other => parent.join(other),
    })
}

fn module_path_override(module: &ItemMod) -> Option<String> {
    module.attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("path") {
            return None;
        }

        match &attr.meta {
            Meta::NameValue(meta) => {
                if let syn::Expr::Lit(expr_lit) = &meta.value
                    && let syn::Lit::Str(path) = &expr_lit.lit
                {
                    return Some(path.value());
                }
                None
            }
            _ => None,
        }
    })
}

fn filter_inline_module(mut module: ItemMod) -> ItemMod {
    if let Some((brace, items)) = module.content.take() {
        let filtered_items = items.into_iter().filter_map(filter_item).collect();
        module.content = Some((brace, filtered_items));
    }
    module
}

fn filter_item(item: Item) -> Option<Item> {
    if is_test_item(&item) {
        return None;
    }

    match item {
        Item::Mod(module) => Some(Item::Mod(filter_inline_module(module))),
        other => Some(other),
    }
}

fn render_file(file: &File) -> String {
    prettyplease::unparse(file).trim().to_owned()
}

fn render_item(item: Item) -> String {
    let file = File {
        shebang: None,
        attrs: vec![],
        items: vec![item],
    };
    render_file(&file)
}

fn is_test_item(item: &Item) -> bool {
    if item_attrs(item).iter().any(is_test_attr) {
        return true;
    }

    matches!(item, Item::Mod(module) if module.ident == "tests")
}

fn is_test_attr(attr: &Attribute) -> bool {
    if matches!(attr.path().segments.last(), Some(seg) if seg.ident == "test") {
        return true;
    }

    if let Meta::List(list) = &attr.meta
        && list.path.is_ident("cfg")
    {
        return cfg_tokens_contain_test(list.tokens.clone());
    }

    false
}

fn cfg_tokens_contain_test(tokens: proc_macro2::TokenStream) -> bool {
    let mut iter = tokens.into_iter().peekable();
    while let Some(token) = iter.next() {
        match &token {
            proc_macro2::TokenTree::Ident(ident) if *ident == "test" => return true,
            proc_macro2::TokenTree::Ident(ident) if *ident == "not" => {
                if matches!(iter.peek(), Some(proc_macro2::TokenTree::Group(_))) {
                    iter.next();
                }
            }
            proc_macro2::TokenTree::Group(group) if cfg_tokens_contain_test(group.stream()) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn item_matches_name(item: &Item, name: &str) -> bool {
    match item {
        Item::Const(item) => item.ident == name,
        Item::Enum(item) => item.ident == name,
        Item::ExternCrate(item) => item.ident == name,
        Item::Fn(item) => item.sig.ident == name,
        Item::Macro(item) => item.ident.as_ref().is_some_and(|ident| ident == name),
        Item::Mod(item) => item.ident == name,
        Item::Static(item) => item.ident == name,
        Item::Struct(item) => item.ident == name,
        Item::Trait(item) => item.ident == name,
        Item::TraitAlias(item) => item.ident == name,
        Item::Type(item) => item.ident == name,
        Item::Union(item) => item.ident == name,
        _ => false,
    }
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

fn use_tree_contains_name(tree: &UseTree, name: &str) -> bool {
    match tree {
        UseTree::Path(path) => path.ident == name || use_tree_contains_name(&path.tree, name),
        UseTree::Name(use_name) => use_name.ident == name,
        UseTree::Rename(rename) => rename.rename == name || rename.ident == name,
        UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| use_tree_contains_name(item, name)),
        UseTree::Glob(_) => false,
    }
}

fn find_use_target(tree: &UseTree, matched_name: &str, prefix: &[String]) -> Option<Vec<String>> {
    match tree {
        UseTree::Path(path) => {
            let mut next_prefix = prefix.to_vec();
            next_prefix.push(path.ident.to_string());
            if path.ident == matched_name {
                Some(next_prefix)
            } else {
                find_use_target(&path.tree, matched_name, &next_prefix)
            }
        }
        UseTree::Name(use_name) => {
            if use_name.ident == matched_name {
                let mut target = prefix.to_vec();
                target.push(use_name.ident.to_string());
                Some(target)
            } else {
                None
            }
        }
        UseTree::Rename(rename) => {
            if rename.rename == matched_name || rename.ident == matched_name {
                let mut target = prefix.to_vec();
                target.push(rename.ident.to_string());
                Some(target)
            } else {
                None
            }
        }
        UseTree::Group(group) => group
            .items
            .iter()
            .find_map(|item| find_use_target(item, matched_name, prefix)),
        UseTree::Glob(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_reexport_target, resolve_rust_path};
    use crate::module_parser::test_utils::TempDirExt;
    use tempfile::TempDir;

    #[test]
    fn resolves_nested_module_and_filters_tests() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "src/lib.rs",
            r"
            pub mod sync;

            #[cfg(test)]
            mod tests {
                #[test]
                fn root_test() {}
            }
            ",
        );
        temp_dir.write(
            "src/sync.rs",
            r"
            pub struct Mutex;

            #[cfg(test)]
            mod tests {
                use super::Mutex;

                #[test]
                fn smoke() {
                    let _ = Mutex;
                }
            }
            ",
        );

        let resolved = resolve_rust_path(&temp_dir.path().join("src/lib.rs"), &["sync"])
            .expect("module should resolve");

        assert_eq!(resolved.source_path, temp_dir.path().join("src/sync.rs"));
        assert_eq!(resolved.source, "pub struct Mutex;");
    }

    #[test]
    fn resolves_reexported_use_items() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "src/lib.rs",
            r"
            pub use macros::module;

            mod macros {
                pub fn module() {}
            }
            ",
        );

        let resolved = resolve_rust_path(&temp_dir.path().join("src/lib.rs"), &["module"])
            .expect("re-export should resolve");

        assert_eq!(resolved.source, "pub use macros::module;");
    }

    #[test]
    fn prefers_real_module_over_matching_reexport() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "src/lib.rs",
            r"
            pub use macros::lifecycle;
            pub mod lifecycle;

            mod macros {
                pub fn lifecycle() {}
            }
            ",
        );
        temp_dir.write(
            "src/lifecycle.rs",
            r"
            pub struct Lifecycle;
            ",
        );

        let resolved = resolve_rust_path(&temp_dir.path().join("src/lib.rs"), &["lifecycle"])
            .expect("module should resolve");

        assert_eq!(
            resolved.source_path,
            temp_dir.path().join("src/lifecycle.rs")
        );
        assert_eq!(resolved.source, "pub struct Lifecycle;");
    }

    #[test]
    fn extracts_reexport_target_for_leaf_match() {
        let target = extract_reexport_target("pub use serde_spanned::Spanned;", "Spanned")
            .expect("extract should parse")
            .expect("target should exist");

        assert_eq!(target, vec!["serde_spanned", "Spanned"]);
    }

    #[test]
    fn extracts_reexport_target_for_leading_segment_match() {
        let target = extract_reexport_target("pub use serde_spanned::Spanned;", "serde_spanned")
            .expect("extract should parse")
            .expect("target should exist");

        assert_eq!(target, vec!["serde_spanned"]);
    }

    #[test]
    fn preserves_cfg_not_test_items() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "src/lib.rs",
            r"
            pub mod utils;
            ",
        );
        temp_dir.write(
            "src/utils.rs",
            r"
            pub fn always() -> bool { true }

            #[cfg(not(test))]
            pub fn prod_only() -> bool { true }

            #[cfg(test)]
            fn test_only() -> bool { false }
            ",
        );

        let resolved = resolve_rust_path(&temp_dir.path().join("src/lib.rs"), &["utils"])
            .expect("module should resolve");

        assert_eq!(
            resolved.source,
            "pub fn always() -> bool {
    true
}
#[cfg(not(test))]
pub fn prod_only() -> bool {
    true
}",
        );
    }

    #[test]
    fn filters_path_qualified_test_attrs() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "src/lib.rs",
            r"
            pub mod handler;
            ",
        );
        temp_dir.write(
            "src/handler.rs",
            r"
            pub async fn handle() {}

            #[tokio::test]
            async fn test_handle() {}
            ",
        );

        let resolved = resolve_rust_path(&temp_dir.path().join("src/lib.rs"), &["handler"])
            .expect("module should resolve");

        assert_eq!(resolved.source, "pub async fn handle() {}");
    }

    #[test]
    fn filters_cfg_all_test() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "src/lib.rs",
            r"
            pub mod mixed;
            ",
        );
        temp_dir.write(
            "src/mixed.rs",
            r#"
            pub struct Prod;

            #[cfg(all(test, feature = "test-utils"))]
            pub struct TestHelper;

            #[cfg_attr(test, derive(Debug))]
            pub struct Both;
            "#,
        );

        let resolved = resolve_rust_path(&temp_dir.path().join("src/lib.rs"), &["mixed"])
            .expect("module should resolve");

        assert_eq!(
            resolved.source,
            "pub struct Prod;
#[cfg_attr(test, derive(Debug))]
pub struct Both;",
        );
    }
}
