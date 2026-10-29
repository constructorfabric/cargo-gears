use crate::common::cargo_cmd;
use crate::packages::PackageScope;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

mod dylint;

pub use dylint::pin_lints;

#[derive(Debug, Eq, PartialEq)]
pub struct LintParams {
    /// Resolved workspace root path.
    pub workspace_root: PathBuf,
    /// Check whether the workspace is formatted with `cargo fmt`.
    pub fmt: bool,
    /// Run recommended clippy rules. Follows Cargo.toml exceptions if present.
    pub clippy: bool,
    /// Strict mode. Throws an error if any lint rule is triggered.
    pub strict: bool,
    /// Run extra lint rules made for gears modules.
    pub dylint: bool,
    /// Lint names to skip when running dylint.
    pub dylint_skip: Vec<String>,
    /// Workspace-wide or explicitly selected package scope.
    pub package_scope: PackageScope,
    /// Expand explicitly selected packages to also include every workspace crate that depends on
    /// them (their reverse-dependency closure) before linting.
    pub include_dependents: bool,
    /// Require Cargo.lock is up to date.
    pub locked: bool,
    /// Cargo feature selection used by Clippy and Dylint.
    pub features: LintFeatureSelection,
    /// List available lints instead of running them.
    pub list: bool,
    /// Pin `cargo-gears-lints` in the workspace's Dylint metadata instead of running lints.
    pub pin_dylint: bool,
}

/// Cargo feature selection for a lint run.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LintFeatureSelection {
    /// Use each selected package's default features.
    Default,
    /// Enable every feature (`--all-features`).
    All,
    /// Enable selected features, optionally without default features.
    Selected {
        /// Feature names passed to Cargo.
        features: Vec<String>,
        /// Whether to pass `--no-default-features`.
        no_default_features: bool,
    },
}

impl LintFeatureSelection {
    fn cargo_args(&self) -> Vec<String> {
        match self {
            Self::Default => Vec::new(),
            Self::All => vec!["--all-features".to_owned()],
            Self::Selected {
                features,
                no_default_features,
            } => {
                let mut args = Vec::new();
                if *no_default_features {
                    args.push("--no-default-features".to_owned());
                }
                if !features.is_empty() {
                    args.push("--features".to_owned());
                    args.push(features.join(","));
                }
                args
            }
        }
    }
}

/// Metadata for a single embedded dylint rule.
#[derive(Debug, Clone)]
pub struct DylintLintInfo {
    /// Lint code, e.g. "DE0101".
    pub code: &'static str,
    /// Rustc-level lint name, e.g. `de0101_no_serde_in_contract`.
    pub name: &'static str,
    /// Category grouping, e.g. "Domain Layer".
    pub category: &'static str,
    /// One-line description of the lint.
    pub description: &'static str,
    /// Default lint level ("deny" or "warn").
    pub default_level: &'static str,
}

/// All embedded dylint rules, sorted by code.
pub static DYLINT_LINTS: &[DylintLintInfo] = &[
    DylintLintInfo {
        code: "DE0101",
        name: "de0101_no_serde_in_contract",
        category: "Domain Layer",
        description: "domain models should not have serde derives",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0102",
        name: "de0102_no_toschema_in_contract",
        category: "Domain Layer",
        description: "domain models should not have ToSchema derive",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0104",
        name: "de0104_no_api_dto_in_contract",
        category: "Domain Layer",
        description: "domain models should not use api_dto macro",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0201",
        name: "de0201_dtos_only_in_api_rest",
        category: "API Layer",
        description: "DTO types should only be defined in */api/rest/* files",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0202",
        name: "de0202_dtos_not_referenced_outside_api",
        category: "API Layer",
        description: "DTO types should not be imported outside of api layer",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0203",
        name: "de0203_dtos_must_use_api_dto",
        category: "API Layer",
        description: "DTO types must use the api_dto macro",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0204",
        name: "de0204_dtos_must_have_toschema_derive",
        category: "API Layer",
        description: "DTO types must derive ToSchema for OpenAPI documentation",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0301",
        name: "de0301_no_infra_in_domain",
        category: "Domain Boundaries",
        description: "domain modules should not import infrastructure dependencies",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0308",
        name: "de0308_no_http_in_domain",
        category: "Domain Boundaries",
        description: "domain modules should not reference HTTP types or status codes",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0309",
        name: "de0309_must_have_domain_model",
        category: "Domain Boundaries",
        description: "domain types must have #[domain_model] attribute",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0503",
        name: "de0503_plugin_client_suffix",
        category: "Client Layer",
        description: "plugin client traits should use *PluginClient suffix",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0504",
        name: "de0504_client_versioning",
        category: "Client Layer",
        description: "Client/PluginClient traits must have version suffixes (V1, V2, ...)",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0706",
        name: "de0706_no_direct_sqlx",
        category: "Security",
        description: "direct sqlx usage is prohibited; use Sea-ORM or SecORM instead",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0707",
        name: "de0707_drop_zeroize",
        category: "Security",
        description: "manual byte-zeroing in Drop may be optimized away; use zeroize crate",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0708",
        name: "de0708_no_non_fips_hasher",
        category: "Security",
        description: "non-FIPS-validated hasher import outside allow-list",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0801",
        name: "de0801_api_endpoint_version",
        category: "REST API Conventions",
        description: "API endpoints must follow /{service-name}/v{N}/{resource} format",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0802",
        name: "de0802_use_odata_ext",
        category: "REST API Conventions",
        description: "use OperationBuilderODataExt instead of .query_param() for OData",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0803",
        name: "de0803_api_snake_case",
        category: "REST API Conventions",
        description: "API DTOs must use snake_case in serde rename attributes",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0901",
        name: "de0901_gts_string_pattern",
        category: "GTS Layer",
        description: "invalid GTS string pattern",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0902",
        name: "de0902_no_schema_for_on_gts_structs",
        category: "GTS Layer",
        description: "GTS structs must use gts_schema_with_refs_as_string() instead of schema_for!()",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE0904",
        name: "de0904_no_hardcoded_gts_prefix",
        category: "GTS Layer",
        description: "hard-coded GTS ID prefix; use gts_id!(\"<suffix>\") instead",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE1101",
        name: "de1101_tests_in_separate_files",
        category: "Testing",
        description: "tests must live in separate files, not inline in production files",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE1201",
        name: "de1201_docs_rs_all_features",
        category: "Documentation",
        description: "crates with features must set docs.rs all-features metadata",
        default_level: "warn",
    },
    DylintLintInfo {
        code: "DE1202",
        name: "de1202_missing_docs_for_pub_crate",
        category: "Documentation",
        description: "crate-public APIs must have documentation",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE1301",
        name: "de1301_no_print_macros",
        category: "Common Patterns",
        description: "print/debug macros are forbidden in production code",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE1302",
        name: "de1302_error_from_to_string",
        category: "Common Patterns",
        description: "calling .to_string() in From<XxxError> impl destroys the error chain",
        default_level: "deny",
    },
    DylintLintInfo {
        code: "DE1303",
        name: "de1303_no_primitive_type_alias",
        category: "Common Patterns",
        description: "pub type X = primitive is a transparent alias; use a newtype",
        default_level: "deny",
    },
];

impl LintParams {
    pub fn run(&self) -> Result<()> {
        if self.list {
            list_lints(self.dylint);
            return Ok(());
        }

        if self.pin_dylint {
            return pin_lints(&self.workspace_root);
        }

        let package_scope = self.effective_package_scope()?;

        if self.fmt {
            run_fmt(&self.workspace_root, &package_scope)?;
        }

        if self.clippy {
            run_clippy(
                &self.workspace_root,
                self.strict,
                &package_scope,
                self.locked,
                &self.features,
            )?;
        }

        if self.dylint {
            dylint::run(
                &self.workspace_root,
                &self.dylint_skip,
                &package_scope,
                self.locked,
                &self.features.cargo_args(),
            )?;
        }

        Ok(())
    }

    /// Apply reverse-dependency expansion to an explicit package scope.
    fn effective_package_scope(&self) -> Result<PackageScope> {
        match &self.package_scope {
            PackageScope::Workspace => Ok(PackageScope::Workspace),
            PackageScope::Selected(packages) if self.include_dependents => {
                let packages =
                    crate::packages::expand_with_dependents(&self.workspace_root, packages)?;
                PackageScope::from_selected(packages)
            }
            PackageScope::Selected(packages) => PackageScope::from_selected(packages.clone()),
        }
    }
}

fn list_lints(dylint_only: bool) {
    if !dylint_only {
        println!("Built-in lint suites:");
        println!("  fmt     Run `cargo fmt --check` for the selected package scope");
        println!("  clippy  Run `cargo clippy --workspace --all-targets`");
        println!("  dylint  Run architectural lint rules (see below)");
        println!();
    }

    println!("Dylint rules ({} total):\n", DYLINT_LINTS.len());

    // Group by category for readability.
    let mut current_category = "";
    for lint in DYLINT_LINTS {
        if lint.category != current_category {
            if !current_category.is_empty() {
                println!();
            }
            println!("  {}:", lint.category);
            current_category = lint.category;
        }
        println!(
            "    {code:<8} {name:<45} {desc}",
            code = lint.code,
            name = lint.name,
            desc = lint.description,
        );
    }
}

fn run_fmt(workspace_path: &Path, package_scope: &PackageScope) -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.args(fmt_cargo_args(package_scope));
    cmd.current_dir(workspace_path);

    let status = cmd.status().context("failed to run `cargo fmt --check`")?;
    if !status.success() {
        anyhow::bail!("`cargo fmt --check` failed with exit status {status}");
    }

    Ok(())
}

fn fmt_cargo_args(package_scope: &PackageScope) -> Vec<String> {
    let mut args = vec!["fmt".to_owned(), "--check".to_owned()];
    match package_scope {
        PackageScope::Workspace => args.push("--all".to_owned()),
        PackageScope::Selected(packages) => {
            for package in packages {
                args.extend(["--package".to_owned(), package.clone()]);
            }
        }
    }
    args
}

fn run_clippy(
    workspace_path: &Path,
    strict: bool,
    package_scope: &PackageScope,
    locked: bool,
    features: &LintFeatureSelection,
) -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.arg("clippy");
    match package_scope {
        PackageScope::Workspace => {
            cmd.arg("--workspace");
        }
        PackageScope::Selected(packages) => {
            for package in packages {
                cmd.args(["--package", package]);
            }
        }
    }
    cmd.arg("--all-targets");
    if locked {
        cmd.arg("--locked");
    }
    cmd.args(features.cargo_args());
    cmd.current_dir(workspace_path);

    // TODO Analyse the manifest feature-set policy and lint those combinations.

    if strict {
        cmd.arg("--").arg("-D").arg("warnings");
    }

    let status = cmd.status().context("failed to run `cargo clippy`")?;
    if !status.success() {
        anyhow::bail!("`cargo clippy` failed with exit status {status}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DYLINT_LINTS, LintFeatureSelection};
    use crate::packages::PackageScope;
    use regex::Regex;
    use std::fs;
    use std::path::Path;

    #[test]
    fn fmt_uses_all_packages_without_a_package_selection() {
        assert_eq!(
            super::fmt_cargo_args(&PackageScope::Workspace),
            ["fmt", "--check", "--all"]
        );
    }

    #[test]
    fn fmt_uses_each_selected_package() {
        assert_eq!(
            super::fmt_cargo_args(&PackageScope::Selected(vec![
                "crate-a".to_owned(),
                "crate-b".to_owned(),
            ])),
            [
                "fmt",
                "--check",
                "--package",
                "crate-a",
                "--package",
                "crate-b",
            ]
        );
    }

    #[test]
    fn lint_feature_selection_maps_to_cargo_arguments() {
        assert_eq!(
            LintFeatureSelection::Default.cargo_args(),
            Vec::<String>::new()
        );
        assert_eq!(LintFeatureSelection::All.cargo_args(), ["--all-features"]);
        assert_eq!(
            LintFeatureSelection::Selected {
                features: vec!["otel".to_owned(), "metrics".to_owned()],
                no_default_features: false,
            }
            .cargo_args(),
            ["--features", "otel,metrics"]
        );
        assert_eq!(
            LintFeatureSelection::Selected {
                features: vec!["sqlite".to_owned()],
                no_default_features: true,
            }
            .cargo_args(),
            ["--no-default-features", "--features", "sqlite"]
        );
        assert_eq!(
            LintFeatureSelection::Selected {
                features: Vec::new(),
                no_default_features: true,
            }
            .cargo_args(),
            ["--no-default-features"]
        );
    }

    #[test]
    fn dylint_lints_registry_is_sorted_by_code() {
        for pair in DYLINT_LINTS.windows(2) {
            assert!(
                pair[0].code < pair[1].code,
                "DYLINT_LINTS not sorted: {} should come before {}",
                pair[0].code,
                pair[1].code,
            );
        }
    }

    #[test]
    fn dylint_lints_names_match_codes() {
        for lint in DYLINT_LINTS {
            let lower_code = lint.code.to_lowercase();
            assert!(
                lint.name.starts_with(&lower_code),
                "lint name `{}` should start with its lowercase code `{}`",
                lint.name,
                lower_code,
            );
        }
    }

    /// `--list` prints this registry, so it must match the lints the pinned
    /// `cargo-gears-lints` declares.
    #[test]
    fn dylint_lints_registry_matches_lints_crate() {
        let lints_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cargo-gears-lints/src");
        let declaration = Regex::new(r"(?m)^\s*pub (DE\d{4}_\w+),\s*\n\s*(Allow|Warn|Deny),")
            .expect("valid regex");

        let mut declared = Vec::new();
        let mut dirs = vec![lints_src];
        while let Some(dir) = dirs.pop() {
            for entry in fs::read_dir(&dir).expect("read lints source dir") {
                let path = entry.expect("read lints source entry").path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    let source = fs::read_to_string(&path).expect("read lint source");
                    declared.extend(
                        declaration.captures_iter(&source).map(|captures| {
                            (captures[1].to_lowercase(), captures[2].to_lowercase())
                        }),
                    );
                }
            }
        }
        declared.sort();

        let registered = DYLINT_LINTS
            .iter()
            .map(|lint| (lint.name.to_owned(), lint.default_level.to_owned()))
            .collect::<Vec<_>>();

        assert_eq!(registered, declared);
    }

    #[test]
    fn list_lints_does_not_panic() {
        super::list_lints(true);
        super::list_lints(false);
    }
}
