use crate::common::cargo_cmd;
use anyhow::{Context, Result};

#[cfg(feature = "dylint-rules")]
use std::collections::BTreeSet;
#[cfg(feature = "dylint-rules")]
use std::fs;
#[cfg(feature = "dylint-rules")]
use std::io::ErrorKind;
#[cfg(feature = "dylint-rules")]
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(feature = "dylint-rules")]
mod ensure_toolchain_installed_shared {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shared/ensure_toolchain_installed.rs"
    ));
}

#[cfg(feature = "dylint-rules")]
use ensure_toolchain_installed_shared::ensure_toolchain_installed;

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
    /// Restrict linting to these packages. Empty means the whole workspace.
    pub packages: Vec<String>,
    /// Expand `packages` to also include every workspace crate that depends on
    /// them (their reverse-dependency closure) before linting.
    pub include_dependents: bool,
    /// Require Cargo.lock is up to date.
    pub locked: bool,
    /// List available lints instead of running them.
    pub list: bool,
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

#[cfg(feature = "dylint-rules")]
include!(concat!(env!("OUT_DIR"), "/generated_libs.rs"));

impl LintParams {
    pub fn run(&self) -> Result<()> {
        if self.list {
            list_lints(self.dylint);
            return Ok(());
        }

        if self.fmt {
            run_fmt(&self.workspace_root)?;
        }

        let packages = self.effective_packages()?;

        if self.clippy {
            run_clippy(&self.workspace_root, self.strict, &packages, self.locked)?;
        }

        if self.dylint {
            run_dylint(
                &self.workspace_root,
                &self.dylint_skip,
                &packages,
                self.locked,
            )?;
        }

        Ok(())
    }

    /// Resolve the package set to lint, applying reverse-dependency expansion
    /// when `include_dependents` is set and at least one package was selected.
    fn effective_packages(&self) -> Result<Vec<String>> {
        if self.include_dependents && !self.packages.is_empty() {
            crate::packages::expand_with_dependents(&self.workspace_root, &self.packages)
        } else {
            Ok(self.packages.clone())
        }
    }
}

fn list_lints(dylint_only: bool) {
    if !dylint_only {
        println!("Built-in lint suites:");
        println!("  fmt     Run `cargo fmt --check --all`");
        println!("  clippy  Run `cargo clippy --workspace --all-targets`");
        println!("  dylint  Run embedded architectural lint rules (see below)");
        println!();
    }

    println!("Embedded dylint rules ({} total):\n", DYLINT_LINTS.len());

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

fn run_fmt(workspace_path: &Path) -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.args(["fmt", "--check", "--all"]);
    cmd.current_dir(workspace_path);

    let status = cmd.status().context("failed to run `cargo fmt --check`")?;
    if !status.success() {
        anyhow::bail!("`cargo fmt --check` failed with exit status {status}");
    }

    Ok(())
}

fn run_clippy(
    workspace_path: &Path,
    strict: bool,
    packages: &[String],
    locked: bool,
) -> Result<()> {
    let mut cmd = cargo_cmd()?;
    cmd.arg("clippy");
    if packages.is_empty() {
        cmd.arg("--workspace");
    } else {
        for package in packages {
            cmd.args(["--package", package]);
        }
    }
    cmd.arg("--all-targets");
    if locked {
        cmd.arg("--locked");
    }
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
fn embedded_toolchains() -> Result<BTreeSet<String>> {
    LIBS.iter()
        .map(|(filename, _)| {
            let (_, toolchain_and_ext) = filename
                .rsplit_once('@')
                .with_context(|| format!("missing toolchain marker in `{filename}`"))?;
            let (toolchain, _) = toolchain_and_ext
                .rsplit_once('.')
                .with_context(|| format!("missing library extension in `{filename}`"))?;
            Ok(toolchain.to_owned())
        })
        .collect()
}

#[cfg(feature = "dylint-rules")]
fn run_dylint(
    workspace_path: &Path,
    skipped_lints: &[String],
    packages: &[String],
    locked: bool,
) -> Result<()> {
    for toolchain in embedded_toolchains()? {
        ensure_toolchain_installed(&toolchain)?;
        clear_dylint_rustc_info_cache(workspace_path, &toolchain)?;
    }

    // Write every embedded dylib to a per-run temp directory so dylint can
    // dlopen them. The temp dir (and its contents) is removed when `tmp_dir`
    // drops at the end of this function, which is safe because `dylint::run`
    // is synchronous and has already finished using the files by then.
    let tmp_dir = tempfile::tempdir().context("could not create temp dir for dylibs")?;

    let lib_paths: Vec<String> = LIBS
        .iter()
        .map(|(filename, bytes)| {
            let dest = tmp_dir.path().join(filename);
            let mut f = std::fs::File::create(&dest)
                .with_context(|| format!("could not create {filename} in temp dir"))?;
            f.write_all(bytes)
                .with_context(|| format!("could not write {filename} to temp dir"))?;
            Ok(dest.to_string_lossy().into_owned())
        })
        .collect::<Result<_>>()?;

    let opts = dylint::opts::Dylint {
        operation: dylint::opts::Operation::Check(dylint::opts::Check {
            lib_sel: dylint::opts::LibrarySelection {
                // Point directly at the extracted, versioned dylib files.
                // dylint parses the toolchain from each filename so no further
                // discovery or building is necessary.
                lib_paths,
                // Check all packages in the workspace rooted at `workspace_path`.
                // Pointing Dylint at the workspace manifest avoids depending on
                // the process CWD.
                manifest_path: Some(
                    workspace_path
                        .join("Cargo.toml")
                        .to_string_lossy()
                        .into_owned(),
                ),
                ..Default::default()
            },
            // Lint the whole workspace unless specific packages were requested
            // on the command line, in which case only those are checked.
            workspace: packages.is_empty(),
            packages: packages.to_vec(),
            args: dylint_cargo_check_args(skipped_lints, locked)?,
            ..Default::default()
        }),
        ..Default::default()
    };

    dylint::run(&opts)
}

#[cfg(feature = "dylint-rules")]
fn dylint_cargo_check_args(skipped_lints: &[String], locked: bool) -> Result<Vec<String>> {
    let mut args = Vec::new();

    if !skipped_lints.is_empty() {
        let rustflags = skipped_lints
            .iter()
            .flat_map(|lint| ["-A".to_owned(), lint.clone()])
            .collect::<Vec<_>>();
        let rustflags =
            serde_json::to_string(&rustflags).context("failed to encode dylint skips")?;
        args.push("--config".to_owned());
        args.push(format!("build.rustflags={rustflags}"));
    }

    if locked {
        args.push("--locked".to_owned());
    }

    Ok(args)
}

#[cfg(feature = "dylint-rules")]
fn clear_dylint_rustc_info_cache(workspace_path: &Path, toolchain: &str) -> Result<()> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(workspace_path.join("Cargo.toml"))
        .no_deps()
        .exec()
        .context("failed to resolve workspace metadata for dylint target dir")?;

    let rustc_info = metadata
        .target_directory
        .as_std_path()
        .join("dylint/target")
        .join(toolchain)
        .join(".rustc_info.json");

    match fs::remove_file(&rustc_info) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| {
            format!(
                "failed to clear stale dylint rustc info cache at {}",
                rustc_info.display()
            )
        }),
    }
}

#[cfg(not(feature = "dylint-rules"))]
fn run_dylint(
    _workspace_path: &Path,
    _skipped_lints: &[String],
    _packages: &[String],
    _locked: bool,
) -> Result<()> {
    anyhow::bail!("dylint-rules feature not enabled")
}

#[cfg(test)]
mod tests {
    use super::DYLINT_LINTS;

    #[cfg(feature = "dylint-rules")]
    #[test]
    fn dylint_skip_list_is_converted_to_cargo_rustflags_config() {
        let args = super::dylint_cargo_check_args(
            &[
                "de0301_no_infra_in_domain".to_owned(),
                "de1302_error_from_to_string".to_owned(),
            ],
            false,
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
