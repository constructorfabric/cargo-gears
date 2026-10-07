use crate::common::cargo_cmd;
use crate::packages::PackageScope;
use anyhow::{Context, Result};

#[cfg(feature = "dylint-rules")]
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
#[cfg(feature = "dylint-rules")]
use std::process::Command;

/// Repository Dylint fetches `cargo-gears-lints` from.
#[cfg(feature = "dylint-rules")]
const LINTS_GIT_URL: &str = "https://github.com/constructorfabric/cargo-gears";

/// Location of the `cargo-gears-lints` package inside [`LINTS_GIT_URL`].
#[cfg(feature = "dylint-rules")]
const LINTS_PATTERN: &str = "crates/cargo-gears-lints";

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
            run_dylint(
                &self.workspace_root,
                &self.dylint_skip,
                &package_scope,
                self.locked,
                &self.features,
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

#[cfg(feature = "dylint-rules")]
fn run_dylint(
    workspace_path: &Path,
    skipped_lints: &[String],
    package_scope: &PackageScope,
    locked: bool,
    features: &LintFeatureSelection,
) -> Result<()> {
    ensure_dylint_link_installed()?;

    // Check all packages in the workspace rooted at `workspace_path`. Pointing
    // Dylint at the workspace manifest avoids depending on the process CWD.
    let manifest_path = Some(
        workspace_path
            .join("Cargo.toml")
            .to_string_lossy()
            .into_owned(),
    );

    // Dylint builds and caches the library with the toolchain pinned by its
    // `rust-toolchain.toml`. Naming a path or git source also makes Dylint
    // ignore any libraries the workspace declares.
    let lib_sel = if let Some(lints_dir) = local_lints_dir() {
        eprintln!("Using cargo-gears-lints from {}", lints_dir.display());
        dylint::opts::LibrarySelection {
            paths: vec![lints_dir.to_string_lossy().into_owned()],
            manifest_path,
            ..Default::default()
        }
    } else {
        // Fetch the lints from the commit this CLI was released from.
        dylint::opts::LibrarySelection {
            git: Some(LINTS_GIT_URL.to_owned()),
            tag: Some(format!("cargo-gears-v{}", env!("CARGO_PKG_VERSION"))),
            pattern: Some(LINTS_PATTERN.to_owned()),
            manifest_path,
            ..Default::default()
        }
    };

    let opts = dylint::opts::Dylint {
        operation: dylint::opts::Operation::Check(dylint::opts::Check {
            lib_sel,
            // Lint the whole workspace unless specific packages were requested
            // on the command line, in which case only those are checked.
            workspace: matches!(package_scope, PackageScope::Workspace),
            packages: package_scope.selected().unwrap_or_default().to_vec(),
            args: dylint_cargo_check_args(skipped_lints, locked, features)?,
            ..Default::default()
        }),
        ..Default::default()
    };

    dylint::run(&opts)
}

/// The `cargo-gears-lints` package next to this crate's sources. It exists
/// when the CLI is built from a clone of the repository or installed with
/// `cargo install --git`, but not when it is installed from crates.io.
#[cfg(feature = "dylint-rules")]
fn local_lints_dir() -> Option<PathBuf> {
    let lints_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .join("cargo-gears-lints");
    lints_dir.join("Cargo.toml").is_file().then_some(lints_dir)
}

#[cfg(feature = "dylint-rules")]
fn dylint_cargo_check_args(
    skipped_lints: &[String],
    locked: bool,
    features: &LintFeatureSelection,
) -> Result<Vec<String>> {
    let mut args = features.cargo_args();

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

/// `cargo-gears-lints` links through `dylint-link` (see its
/// `.cargo/config.toml`), so Dylint cannot build it without that tool.
#[cfg(feature = "dylint-rules")]
fn ensure_dylint_link_installed() -> Result<()> {
    // `dylint-link` forwards its arguments to the system linker, so only
    // whether it can be spawned matters, not its exit status.
    match Command::new("dylint-link").arg("--version").output() {
        Ok(_) => return Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(err).context("failed to run `dylint-link`"),
    }

    eprintln!("Installing dylint-link...");
    let status = cargo_cmd()?
        .args(["install", "--locked", "dylint-link"])
        .status()
        .context("failed to run `cargo install dylint-link`")?;
    if !status.success() {
        anyhow::bail!("`cargo install dylint-link` failed with exit status {status}");
    }

    Ok(())
}

#[cfg(not(feature = "dylint-rules"))]
fn run_dylint(
    _workspace_path: &Path,
    _skipped_lints: &[String],
    _package_scope: &PackageScope,
    _locked: bool,
    _features: &LintFeatureSelection,
) -> Result<()> {
    anyhow::bail!("dylint-rules feature not enabled")
}

#[cfg(test)]
mod tests {
    use super::{DYLINT_LINTS, LintFeatureSelection};
    use crate::packages::PackageScope;

    #[cfg(feature = "dylint-rules")]
    #[test]
    fn dylint_skip_list_is_converted_to_cargo_rustflags_config() {
        let args = super::dylint_cargo_check_args(
            &[
                "de0301_no_infra_in_domain".to_owned(),
                "de1302_error_from_to_string".to_owned(),
            ],
            false,
            &LintFeatureSelection::Default,
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
        assert!(LintFeatureSelection::Default.cargo_args().is_empty());
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

    #[cfg(feature = "dylint-rules")]
    #[test]
    fn dylint_feature_arguments_are_combined_with_skip_config_and_locked() {
        let args = super::dylint_cargo_check_args(
            &["de1301_no_print_macros".to_owned()],
            true,
            &LintFeatureSelection::Selected {
                features: vec!["otel".to_owned()],
                no_default_features: true,
            },
        )
        .expect("dylint arguments should encode");

        assert_eq!(
            args,
            [
                "--no-default-features",
                "--features",
                "otel",
                "--config",
                "build.rustflags=[\"-A\",\"de1301_no_print_macros\"]",
                "--locked",
            ]
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

    #[test]
    fn list_lints_does_not_panic() {
        super::list_lints(true);
        super::list_lints(false);
    }
}
