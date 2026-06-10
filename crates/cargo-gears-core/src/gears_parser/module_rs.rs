use super::config::{Capability, ConfigModule, ConfigModuleMetadata};
use anyhow::{Context, bail};
use cargo_metadata::{Package, Target};
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Item, Lit, Meta};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModule {
    pub name: String,
    pub deps: Vec<String>,
    pub capabilities: Vec<Capability>,
}

pub fn retrieve_gears_module(
    package: &Package,
    target: &Target,
) -> anyhow::Result<(String, ConfigModule)> {
    let lib_rs = PathBuf::from(&target.src_path);
    let src = lib_rs
        .parent()
        .with_context(|| format!("no source parent for {}", target.src_path))?;
    let parsed_module = find_gears_module_in_src(src)
        .with_context(|| format!("no gears module found in {}", src.display()))?;
    let crate_root = PathBuf::from(&package.manifest_path)
        .parent()
        .map(|p| p.display().to_string());

    let config_module = ConfigModule {
        metadata: ConfigModuleMetadata {
            package: Some(package.name.to_string()),
            version: Some(package.version.to_string()),
            features: vec![],
            default_features: None,
            path: crate_root,
            deps: parsed_module.deps,
            capabilities: parsed_module.capabilities,
        },
    };
    Ok((parsed_module.name, config_module))
}

/// Recursively scans all `.rs` files under `src/` for a gears module annotation.
fn find_gears_module_in_src(src: &Path) -> anyhow::Result<ParsedModule> {
    if let Some(parsed) = scan_dir_for_gears_module(src)? {
        return Ok(parsed);
    }
    bail!("no gears module annotation found in {}", src.display())
}

/// Recursively walks `dir`, trying to parse each `.rs` file for a gears module
/// annotation. Returns the first match, or `None` if no annotated struct is found.
fn scan_dir_for_gears_module(dir: &Path) -> anyhow::Result<Option<ParsedModule>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("can't read source directory {}", dir.display()))?;

    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            if let Some(parsed) = scan_dir_for_gears_module(&path)? {
                return Ok(Some(parsed));
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(parsed) = parse_module_rs_source(&content) {
                return Ok(Some(parsed));
            }
        }
    }

    Ok(None)
}

pub fn parse_module_rs_source(content: &str) -> anyhow::Result<ParsedModule> {
    let ast = syn::parse_file(content)?;
    for item in ast.items {
        if let Item::Struct(struct_item) = item
            && let Some(module_info) = parse_gears_module_attribute(&struct_item.attrs)?
        {
            return Ok(ParsedModule {
                name: module_info.name,
                deps: module_info.deps,
                capabilities: module_info.capabilities,
            });
        }
    }

    Err(super::source::NotFoundError("no module found".into()).into())
}

struct ModuleInfo {
    name: String,
    deps: Vec<String>,
    capabilities: Vec<Capability>,
}

fn parse_gears_module_attribute(attrs: &[Attribute]) -> anyhow::Result<Option<ModuleInfo>> {
    for attr in attrs {
        if is_gears_module_path(attr) {
            return parse_module_args(attr).map(Some);
        }
    }
    Ok(None)
}

fn is_gears_module_path(attr: &Attribute) -> bool {
    let path = attr.path();
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();

    (segments.len() == 1 && segments[0] == "module")
        || (segments.len() == 2 && segments[0] == "gears_toolkit" && segments[1] == "module")
}

fn parse_module_args(attr: &Attribute) -> anyhow::Result<ModuleInfo> {
    let mut name = None;
    let mut deps = Vec::new();
    let mut capabilities = Vec::new();

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            let value = meta.value()?;
            let lit: Lit = value.parse()?;
            if let Lit::Str(lit_str) = lit {
                name = Some(lit_str.value());
            }
        } else if meta.path.is_ident("deps") {
            let value = meta.value()?;
            let expr: syn::Expr = value.parse()?;
            if let syn::Expr::Array(array) = expr {
                for element in array.elems {
                    if let syn::Expr::Lit(syn::ExprLit {
                                              lit: Lit::Str(lit_str),
                                              ..
                                          }) = element
                    {
                        deps.push(lit_str.value());
                    }
                }
            }
        } else if meta.path.is_ident("capabilities") {
            let value = meta.value()?;
            let expr: syn::Expr = value.parse()?;
            if let syn::Expr::Array(array) = expr {
                for element in array.elems {
                    let capability = match element {
                        syn::Expr::Path(path_expr) => {
                            let Some(ident) = path_expr.path.get_ident() else {
                                return Err(syn::Error::new_spanned(
                                    path_expr.path,
                                    "capability must be a simple identifier",
                                ));
                            };
                            parse_capability_name(&ident.to_string()).ok_or_else(|| {
                                syn::Error::new_spanned(
                                    ident,
                                    "unknown capability, expected one of: db, rest, rest_host, stateful, system, grpc_hub, grpc",
                                )
                            })?
                        }
                        syn::Expr::Lit(syn::ExprLit {
                                           lit: Lit::Str(lit_str),
                                           ..
                                       }) => parse_capability_name(&lit_str.value()).ok_or_else(|| {
                            syn::Error::new_spanned(
                                lit_str,
                                "unknown capability, expected one of: db, rest, rest_host, stateful, system, grpc_hub, grpc",
                            )
                        })?,
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "capability must be an identifier or string literal",
                            ));
                        }
                    };
                    capabilities.push(capability);
                }
            } else {
                return Err(syn::Error::new_spanned(
                    expr,
                    "capabilities must be an array, e.g. capabilities = [db, rest]",
                ));
            }
        } else {
            consume_unknown_meta(meta.input)?;
        }
        Ok(())
    })?;

    let name = name.context("module attribute must have a name")?;
    Ok(ModuleInfo {
        name,
        deps,
        capabilities,
    })
}

fn parse_capability_name(name: &str) -> Option<Capability> {
    match name {
        "db" => Some(Capability::Db),
        "rest" => Some(Capability::Rest),
        "rest_host" => Some(Capability::RestHost),
        "stateful" => Some(Capability::Stateful),
        "system" => Some(Capability::System),
        "grpc_hub" => Some(Capability::GrpcHub),
        "grpc" => Some(Capability::Grpc),
        _ => None,
    }
}

fn consume_unknown_meta(input: ParseStream<'_>) -> syn::Result<()> {
    if input.peek(syn::Token![=]) {
        let _: syn::Token![=] = input.parse()?;
        let _expr: syn::Expr = input.parse()?;
    } else if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        let _nested: syn::punctuated::Punctuated<Meta, syn::Token![,]> =
            content.parse_terminated(Meta::parse, syn::Token![,])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_module_rs_source;
    use crate::gears_parser::Capability;

    #[test]
    fn parses_module_with_lifecycle_meta() {
        let content = r#"
            #[gears_toolkit::module(
                name = "grpc-hub",
                capabilities = [stateful, system, grpc_hub],
                lifecycle(entry = "serve", await_ready)
            )]
            pub struct GrpcHub;
        "#;

        let parsed = parse_module_rs_source(content).expect("module should parse");
        assert_eq!(parsed.name, "grpc-hub");
        assert!(parsed.deps.is_empty());
        assert_eq!(
            parsed.capabilities,
            vec![
                Capability::Stateful,
                Capability::System,
                Capability::GrpcHub
            ]
        );
    }

    #[test]
    fn parses_deps_list() {
        let content = r#"
            #[module(name = "demo", deps = ["authz", "tenant-resolver"])]
            pub struct Demo;
        "#;

        let parsed = parse_module_rs_source(content).expect("module should parse");
        assert_eq!(parsed.name, "demo");
        assert_eq!(parsed.deps, vec!["authz", "tenant-resolver"]);
        assert!(parsed.capabilities.is_empty());
    }

    #[test]
    fn parses_capabilities_from_strings() {
        let content = r#"
            #[module(name = "demo", capabilities = ["db", "rest_host"])]
            pub struct Demo;
        "#;

        let parsed = parse_module_rs_source(content).expect("module should parse");
        assert_eq!(parsed.name, "demo");
        assert_eq!(
            parsed.capabilities,
            vec![Capability::Db, Capability::RestHost]
        );
    }
}
