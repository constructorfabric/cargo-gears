use crate::common::Registry;
use crate::gears_parser::{
    LibraryMapping, ResolvedMetadataPath, extract_reexport_target,
    list_library_mappings_from_metadata, resolve_source_from_metadata,
};
use anyhow::{Context, bail};
use flate2::read::GzDecoder;
use reqwest::{Client, Method, StatusCode, retry};
use semver::Version;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Resolve Rust source code from a crate
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceParams {
    /// Path to the Cargo workspace or crate to inspect
    pub path: PathBuf,
    /// Registry to query when the crate is not present in local metadata
    pub registry: Registry,
    /// Print query/package/version/source metadata before the resolved Rust source
    pub verbose: bool,
    /// List `library_name` -> `package_name` mappings for a package query
    pub libs: bool,
    /// Resolve a specific crate version after metadata/cache lookup misses
    pub version: Option<Version>,
    /// Remove the source cache for the selected registry before resolving
    pub clean: bool,
    /// Rust path to resolve(start always by `package_name`), for example `cf-gears-toolkit` it will resolve the lib.rs
    /// You can resolve modules `tokio::sync` to resolve the source code from the sync module from tokio crate
    /// You can also resolve by function name, for example `cf-gears-toolkit::gts::plugin::BaseGearsPluginV1`
    /// Also resolve by function name, for instance `cf-gears-toolkit::gts::schemas::get_core_gts_schemas`
    pub query: Option<String>,
}

impl SourceParams {
    pub fn run(&self) -> anyhow::Result<()> {
        if self.clean {
            clean_registry_cache(self.registry)?;
        }

        let Some(query) = self.query.as_deref() else {
            return if self.clean {
                Ok(())
            } else {
                bail!("source query is required unless --clean is used by itself")
            };
        };

        let workspace_path = self
            .path
            .canonicalize()
            .with_context(|| format!("can't canonicalize path {}", self.path.display()))?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to build tokio runtime for source queries")?;
        let client = build_registry_client()?;
        let resolution_ctx = Resolver {
            workspace_path: &workspace_path,
            client: &client,
            runtime: &runtime,
            registry: self.registry,
        };
        let mut visited = HashSet::new();
        let final_resolution = resolve_query_recursive(
            &resolution_ctx,
            &workspace_path,
            query,
            self.version.as_ref(),
            &mut visited,
        )?;

        if self.libs {
            let mappings = list_library_mappings(
                final_resolution.manifest_path.parent(),
                &final_resolution.package_name,
            )?;
            print_library_mappings(query, &final_resolution, &mappings, self.verbose);
            return Ok(());
        }

        print_resolved_path(query, &final_resolution, self.verbose);

        Ok(())
    }
}

fn print_resolved_path(query: &str, resolved: &ResolvedMetadataPath, verbose: bool) {
    if verbose {
        println!("query: {query}");
        println!("package: {}", resolved.package_name);
        println!("library: {}", resolved.library_name);
        println!("version: {}", resolved.version);
        println!("manifest: {}", resolved.manifest_path.display());
        println!("source: {}", resolved.source_path.display());
        println!();
    }

    println!("{}", resolved.source);
}

fn print_library_mappings(
    query: &str,
    resolved: &ResolvedMetadataPath,
    mappings: &[LibraryMapping],
    verbose: bool,
) {
    if verbose {
        println!("query: {query}");
        println!("package: {}", resolved.package_name);
        println!("version: {}", resolved.version);
        println!("manifest: {}", resolved.manifest_path.display());
        println!();
    }

    for mapping in mappings {
        println!("{} -> {}", mapping.library_name, mapping.package_name);
    }
}

fn list_library_mappings(
    crate_root: Option<&Path>,
    query: &str,
) -> anyhow::Result<Vec<LibraryMapping>> {
    let crate_root = crate_root.context("resolved manifest path has no parent")?;
    let package_query = package_only_query(query)?;
    list_library_mappings_from_metadata(crate_root, &package_query)?
        .with_context(|| format!("could not list libraries for package '{package_query}'"))
}

fn package_only_query(query: &str) -> anyhow::Result<String> {
    let segments = split_query_segments(query)?;
    if segments.len() != 1 {
        bail!("--libs requires a package-only query such as 'cf-gears-toolkit'");
    }

    Ok(segments[0].clone())
}

struct RetryAllHosts;

impl PartialEq<&str> for RetryAllHosts {
    fn eq(&self, _: &&str) -> bool {
        true
    }
}

fn should_retry_registry_request(
    method: &Method,
    status: Option<StatusCode>,
    has_error: bool,
) -> bool {
    if method != Method::GET {
        return false;
    }

    if has_error {
        return true;
    }

    status.is_some_and(|status| {
        matches!(
            status,
            StatusCode::REQUEST_TIMEOUT
                | StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        )
    })
}

fn build_registry_client() -> anyhow::Result<Client> {
    Client::builder()
        .user_agent("cargo-gears")
        .timeout(Duration::from_secs(20))
        .retry(retry::for_host(RetryAllHosts).classify_fn(|req_rep| {
            if should_retry_registry_request(
                req_rep.method(),
                req_rep.status(),
                req_rep.error().is_some(),
            ) {
                req_rep.retryable()
            } else {
                req_rep.success()
            }
        }))
        .build()
        .context("failed to create registry HTTP client")
}

struct Resolver<'a> {
    workspace_path: &'a Path,
    client: &'a Client,
    runtime: &'a tokio::runtime::Runtime,
    registry: Registry,
}

fn resolve_query_recursive(
    context: &Resolver<'_>,
    preferred_path: &Path,
    query: &str,
    requested_version: Option<&Version>,
    visited: &mut HashSet<String>,
) -> anyhow::Result<ResolvedMetadataPath> {
    let visit_key = format!(
        "{}|{}|{}",
        preferred_path.display(),
        query,
        requested_version.map_or_else(|| "*".to_owned(), ToString::to_string)
    );
    if !visited.insert(visit_key) {
        bail!("detected recursive re-export loop while resolving '{query}'");
    }

    let Some(resolution) = resolve_from_paths(
        context.workspace_path,
        preferred_path,
        context.client,
        context.runtime,
        context.registry,
        query,
        requested_version,
    )?
    else {
        bail!("could not resolve '{query}'");
    };

    if let Some(next_step) = next_reexport_step(preferred_path, &resolution, query)? {
        return resolve_query_recursive(
            context,
            &next_step.preferred_path,
            &next_step.query,
            next_step.requested_version.as_ref(),
            visited,
        );
    }

    Ok(resolution)
}

fn resolve_from_paths(
    workspace_path: &Path,
    preferred_path: &Path,
    client: &Client,
    runtime: &tokio::runtime::Runtime,
    registry: Registry,
    query: &str,
    requested_version: Option<&Version>,
) -> anyhow::Result<Option<ResolvedMetadataPath>> {
    if let Some(resolved) = resolve_source_from_metadata(preferred_path, query)? {
        return Ok(Some(resolved));
    }

    if preferred_path != workspace_path
        && let Some(resolved) = resolve_source_from_metadata(workspace_path, query)?
    {
        return Ok(Some(resolved));
    }

    runtime
        .block_on(resolve_from_registry(
            client,
            registry,
            query,
            requested_version,
        ))
        .map(Some)
}

async fn resolve_from_registry(
    client: &Client,
    registry: Registry,
    query: &str,
    requested_version: Option<&Version>,
) -> anyhow::Result<ResolvedMetadataPath> {
    let crate_name = query
        .split("::")
        .next()
        .filter(|segment| !segment.is_empty())
        .context("query must not be empty")?;

    if let Some(resolved) = resolve_from_cache(registry, crate_name, query, requested_version)? {
        return Ok(resolved);
    }

    let resolved_version = if let Some(requested_version) = requested_version {
        requested_version.to_string()
    } else {
        fetch_exact_registry_candidate(client, registry, crate_name)
            .await?
            .with_context(|| {
                format!("could not resolve package '{crate_name}' from the {registry} registry")
            })?
            .max_version
    };
    let crate_root = cache_crate_source(client, registry, crate_name, &resolved_version).await?;

    resolve_source_from_metadata(&crate_root, query)?
        .with_context(|| format!("could not resolve '{query}' inside package '{crate_name}'"))
}

struct NextStep {
    preferred_path: PathBuf,
    query: String,
    requested_version: Option<Version>,
}

fn next_reexport_step(
    preferred_path: &Path,
    resolved: &ResolvedMetadataPath,
    query: &str,
) -> anyhow::Result<Option<NextStep>> {
    let Some(target_segments) = extract_reexport_target(
        &resolved.source,
        query
            .split("::")
            .last()
            .context("query must not be empty")?,
    )?
    else {
        return Ok(None);
    };

    let crate_root = resolved
        .manifest_path
        .parent()
        .context("resolved manifest path has no parent")?;
    let package_name = &resolved.package_name;
    let query_segments = split_query_segments(query)?;
    let package_index = query_segments
        .iter()
        .position(|segment| segment == package_name)
        .context("resolved query does not include package name")?;
    let containing_module_segments = query_segments[package_index + 1..]
        .split_last()
        .map_or_else(Vec::new, |(_, module_segments)| module_segments.to_vec());

    if let Some(relative_segments) =
        resolve_relative_reexport(&target_segments, &containing_module_segments)?
    {
        let next_query = build_query(package_name, &relative_segments);
        return Ok(Some(NextStep {
            preferred_path: preferred_path.to_path_buf(),
            query: next_query,
            requested_version: None,
        }));
    }

    let dependencies = parse_dependencies(crate_root)?;

    let Some((first_segment, remaining_segments)) = target_segments.split_first() else {
        return Ok(None);
    };

    let Some(dep) = find_dependency_spec(&dependencies, first_segment) else {
        if let Some(next_query) = resolve_bare_relative_reexport(
            crate_root,
            package_name,
            &target_segments,
            &containing_module_segments,
        )? {
            return Ok(Some(NextStep {
                preferred_path: preferred_path.to_path_buf(),
                query: next_query,
                requested_version: None,
            }));
        }

        let next_query = build_query(package_name, &target_segments);
        return Ok(Some(NextStep {
            preferred_path: preferred_path.to_path_buf(),
            query: next_query,
            requested_version: None,
        }));
    };

    let next_query = build_query(&dep.package_name, remaining_segments);
    let next_preferred_path = dep.path.as_ref().map_or_else(
        || preferred_path.to_path_buf(),
        |path| crate_root.join(path),
    );

    Ok(Some(NextStep {
        preferred_path: next_preferred_path,
        query: next_query,
        requested_version: dep.version.clone(),
    }))
}

fn resolve_bare_relative_reexport(
    crate_root: &Path,
    package_name: &str,
    target_segments: &[String],
    containing_module_segments: &[String],
) -> anyhow::Result<Option<String>> {
    if containing_module_segments.is_empty() {
        return Ok(None);
    }

    let relative_segments = containing_module_segments
        .iter()
        .cloned()
        .chain(target_segments.iter().cloned())
        .collect::<Vec<_>>();
    let relative_query = build_query(package_name, &relative_segments);

    Ok(resolve_source_from_metadata(crate_root, &relative_query)?.map(|_| relative_query))
}

fn split_query_segments(query: &str) -> anyhow::Result<Vec<String>> {
    let segments = query
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if segments.is_empty() {
        bail!("query must not be empty");
    }
    Ok(segments)
}

fn build_query(package_name: &str, segments: &[String]) -> String {
    if segments.is_empty() {
        package_name.to_owned()
    } else {
        format!("{package_name}::{}", segments.join("::"))
    }
}

fn resolve_relative_reexport(
    target_segments: &[String],
    containing_module_segments: &[String],
) -> anyhow::Result<Option<Vec<String>>> {
    let Some(first) = target_segments.first() else {
        return Ok(None);
    };

    match first.as_str() {
        "crate" => Ok(Some(target_segments[1..].to_vec())),
        "self" => Ok(Some(
            containing_module_segments
                .iter()
                .cloned()
                .chain(target_segments[1..].iter().cloned())
                .collect(),
        )),
        "super" => {
            let mut module_segments = containing_module_segments.to_vec();
            let mut index = 0;
            while target_segments
                .get(index)
                .is_some_and(|segment| segment == "super")
            {
                if module_segments.pop().is_none() {
                    bail!("re-export path moves above crate root");
                }
                index += 1;
            }
            Ok(Some(
                module_segments
                    .into_iter()
                    .chain(target_segments[index..].iter().cloned())
                    .collect(),
            ))
        }
        _ => Ok(None),
    }
}

#[derive(Clone)]
struct DependencySpec {
    package_name: String,
    version: Option<Version>,
    path: Option<PathBuf>,
}

fn parse_dependencies(crate_root: &Path) -> anyhow::Result<HashMap<String, DependencySpec>> {
    let manifest_path = crate_root.join("Cargo.toml");
    let manifest = read_manifest(&manifest_path)?;
    let workspace_deps = read_workspace_dependencies(crate_root)?;

    let mut deps = HashMap::new();
    collect_dependency_table(
        &mut deps,
        &manifest,
        "dependencies",
        workspace_deps.as_ref(),
    );
    collect_target_dependency_tables(&mut deps, &manifest, workspace_deps.as_ref());

    Ok(deps)
}

fn read_manifest(manifest_path: &Path) -> anyhow::Result<toml_edit::DocumentMut> {
    let manifest = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;
    manifest
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse manifest {}", manifest_path.display()))
}

fn read_workspace_dependencies(
    crate_root: &Path,
) -> anyhow::Result<Option<HashMap<String, DependencySpec>>> {
    let Some(workspace_manifest_path) = find_workspace_manifest(crate_root)? else {
        return Ok(None);
    };
    let workspace_manifest = read_manifest(&workspace_manifest_path)?;
    let Some(table) = workspace_manifest
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml_edit::Item::as_table_like)
    else {
        return Ok(None);
    };

    let mut deps = HashMap::new();
    for (alias, value) in table.iter() {
        deps.insert(alias.to_owned(), parse_dependency_spec(alias, value));
    }

    Ok(Some(deps))
}

fn find_workspace_manifest(crate_root: &Path) -> anyhow::Result<Option<PathBuf>> {
    for dir in crate_root.ancestors() {
        let manifest_path = dir.join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }

        let manifest = read_manifest(&manifest_path)?;
        if manifest.get("workspace").is_some() {
            return Ok(Some(manifest_path));
        }
    }

    Ok(None)
}

fn collect_dependency_table(
    deps: &mut HashMap<String, DependencySpec>,
    manifest: &toml_edit::DocumentMut,
    table_name: &str,
    workspace_deps: Option<&HashMap<String, DependencySpec>>,
) {
    if let Some(table) = manifest
        .get(table_name)
        .and_then(toml_edit::Item::as_table_like)
    {
        for (alias, value) in table.iter() {
            let spec = parse_dependency_spec_with_workspace(alias, value, workspace_deps);
            deps.insert(alias.to_owned(), spec);
        }
    }
}

fn collect_target_dependency_tables(
    deps: &mut HashMap<String, DependencySpec>,
    manifest: &toml_edit::DocumentMut,
    workspace_deps: Option<&HashMap<String, DependencySpec>>,
) {
    let Some(targets) = manifest
        .get("target")
        .and_then(toml_edit::Item::as_table_like)
    else {
        return;
    };

    for (_, target) in targets.iter() {
        let Some(dependencies) = target
            .as_table_like()
            .and_then(|target| target.get("dependencies"))
            .and_then(toml_edit::Item::as_table_like)
        else {
            continue;
        };

        for (alias, value) in dependencies.iter() {
            let spec = parse_dependency_spec_with_workspace(alias, value, workspace_deps);
            deps.insert(alias.to_owned(), spec);
        }
    }
}

/// Parses a dependency entry from a `Cargo.toml` `[dependencies]` table.
///
/// Only exact three-component semver versions (e.g. `"1.0.210"`) are captured
/// in `DependencySpec::version`. Version *requirements* such as `"^1.0"`,
/// `">=1.0"`, or two-component shorthand like `"1.0"` will not parse as
/// `semver::Version` and are silently treated as `None`. This is intentional:
/// the version field is only used for exact cache lookups and crate downloads,
/// and non-exact requirements cause a fallback to the registry's latest version
/// which is the correct behaviour for doc resolution.
fn parse_dependency_spec(alias: &str, value: &toml_edit::Item) -> DependencySpec {
    if let Some(version) = value.as_str() {
        return DependencySpec {
            package_name: alias.to_owned(),
            version: Version::parse(version).ok(),
            path: None,
        };
    }

    if let Some(table) = value.as_inline_table() {
        return DependencySpec {
            package_name: table
                .get("package")
                .and_then(toml_edit::Value::as_str)
                .unwrap_or(alias)
                .to_owned(),
            version: table
                .get("version")
                .and_then(toml_edit::Value::as_str)
                .and_then(|version| Version::parse(version).ok()),
            path: table
                .get("path")
                .and_then(toml_edit::Value::as_str)
                .map(PathBuf::from),
        };
    }

    if let Some(table) = value.as_table_like() {
        return DependencySpec {
            package_name: table
                .get("package")
                .and_then(toml_edit::Item::as_str)
                .unwrap_or(alias)
                .to_owned(),
            version: table
                .get("version")
                .and_then(toml_edit::Item::as_str)
                .and_then(|version| Version::parse(version).ok()),
            path: table
                .get("path")
                .and_then(toml_edit::Item::as_str)
                .map(PathBuf::from),
        };
    }

    DependencySpec {
        package_name: alias.to_owned(),
        version: None,
        path: None,
    }
}

fn parse_dependency_spec_with_workspace(
    alias: &str,
    value: &toml_edit::Item,
    workspace_deps: Option<&HashMap<String, DependencySpec>>,
) -> DependencySpec {
    let mut spec = parse_dependency_spec(alias, value);
    if dependency_uses_workspace_inheritance(value)
        && let Some(workspace_spec) = workspace_deps.and_then(|deps| deps.get(alias))
    {
        if spec.package_name == alias {
            spec.package_name.clone_from(&workspace_spec.package_name);
        }
        if spec.version.is_none() {
            spec.version.clone_from(&workspace_spec.version);
        }
    }
    spec
}

fn dependency_uses_workspace_inheritance(value: &toml_edit::Item) -> bool {
    get_dep_bool_field(value, "workspace").unwrap_or(false)
}

fn get_dep_value<'a>(dep: &'a toml_edit::Item, key: &str) -> Option<&'a toml_edit::Value> {
    dep.as_table()
        .and_then(|t| t.get(key))
        .and_then(toml_edit::Item::as_value)
        .or_else(|| dep.as_inline_table().and_then(|t| t.get(key)))
}

fn get_dep_bool_field(dep: &toml_edit::Item, key: &str) -> Option<bool> {
    get_dep_value(dep, key).and_then(toml_edit::Value::as_bool)
}

fn find_dependency_spec<'a>(
    dependencies: &'a HashMap<String, DependencySpec>,
    rust_crate_name: &str,
) -> Option<&'a DependencySpec> {
    dependencies.get(rust_crate_name).or_else(|| {
        dependencies
            .iter()
            .find(|(alias, _)| normalize_dependency_alias(alias) == rust_crate_name)
            .map(|(_, spec)| spec)
    })
}

fn normalize_dependency_alias(alias: &str) -> String {
    alias.replace('-', "_")
}

async fn fetch_exact_registry_candidate(
    client: &Client,
    registry: Registry,
    crate_name: &str,
) -> anyhow::Result<Option<ExactCrate>> {
    let crate_url = format!("https://{registry}/api/v1/crates/{crate_name}");
    let response = client
        .get(&crate_url)
        .send()
        .await
        .with_context(|| format!("request failed for '{crate_name}'"))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let response = response
        .error_for_status()
        .with_context(|| format!("registry returned an error for '{crate_name}'"))?
        .json::<ExactCrateResponse>()
        .await
        .with_context(|| format!("invalid crate metadata for '{crate_name}'"))?;

    Ok(Some(ExactCrate {
        max_version: response.crate_info.max_version,
    }))
}

#[derive(Deserialize)]
struct ExactCrateResponse {
    #[serde(rename = "crate")]
    crate_info: ExactCrateInfo,
}

#[derive(Deserialize)]
struct ExactCrateInfo {
    max_version: String,
}

struct ExactCrate {
    max_version: String,
}

async fn download_crate_archive(
    client: &Client,
    registry: Registry,
    crate_name: &str,
    version: &str,
) -> anyhow::Result<Vec<u8>> {
    let download_url = format!("https://{registry}/api/v1/crates/{crate_name}/{version}/download");
    let archive = client
        .get(&download_url)
        .send()
        .await
        .with_context(|| format!("download request failed for {crate_name}"))?
        .error_for_status()
        .with_context(|| format!("download endpoint returned an error for {crate_name}"))?
        .bytes()
        .await
        .with_context(|| format!("failed to read downloaded source for {crate_name}"))?;

    Ok(archive.to_vec())
}

async fn cache_crate_source(
    client: &Client,
    registry: Registry,
    crate_name: &str,
    version: &str,
) -> anyhow::Result<PathBuf> {
    let package_root = package_cache_root(registry, crate_name)?;
    let crate_root = package_root.join(version);

    if crate_root.join("Cargo.toml").is_file() {
        return Ok(crate_root);
    }

    let archive_bytes = download_crate_archive(client, registry, crate_name, version).await?;
    extract_crate_archive(&archive_bytes, &package_root, crate_name, version)?;
    update_latest_symlink(&package_root, version)?;

    if crate_root.join("Cargo.toml").is_file() {
        Ok(crate_root)
    } else {
        bail!("cached crate source is missing Cargo.toml for {crate_name} {version}");
    }
}

fn registry_cache_root(registry: Registry) -> anyhow::Result<PathBuf> {
    let cache_root = std::env::temp_dir()
        .join("gears-docs-cache")
        .join(sanitize_registry_name(registry));
    fs::create_dir_all(&cache_root)
        .with_context(|| format!("failed to create cache dir {}", cache_root.display()))?;
    Ok(cache_root)
}

fn package_cache_root(registry: Registry, crate_name: &str) -> anyhow::Result<PathBuf> {
    let package_root = registry_cache_root(registry)?.join(crate_name);
    fs::create_dir_all(&package_root).with_context(|| {
        format!(
            "failed to create package cache dir {}",
            package_root.display()
        )
    })?;
    Ok(package_root)
}

fn resolve_from_cache(
    registry: Registry,
    crate_name: &str,
    query: &str,
    requested_version: Option<&Version>,
) -> anyhow::Result<Option<ResolvedMetadataPath>> {
    let package_root = package_cache_root(registry, crate_name)?;

    if let Some(requested_version) = requested_version {
        let crate_root = package_root.join(requested_version.to_string());
        return resolve_from_cached_root(&crate_root, query);
    }

    let latest_link = package_root.join("latest");
    if let Some(resolved) = resolve_from_cached_root(&latest_link, query)? {
        return Ok(Some(resolved));
    }

    // NOTE: the symlink update below is not protected against concurrent processes.
    // If multiple `gears src` invocations race here, the symlink may be
    // updated more than once, but the result is still correct (points to the
    // highest cached version). Add file-based locking on `package_root` if this
    // ever becomes a problem in practice.
    let mut cached_versions = cached_package_versions(&package_root)?;
    cached_versions
        .sort_by(|(left_version, _), (right_version, _)| right_version.cmp(left_version));

    if let Some((latest_version, _)) = cached_versions.first() {
        let needs_update = fs::read_link(&latest_link)
            .ok()
            .and_then(|target| target.file_name().map(OsStr::to_os_string))
            .is_none_or(|current| current != latest_version.to_string().as_str());
        if needs_update {
            update_latest_symlink(&package_root, &latest_version.to_string())?;
        }
    }

    for (_, crate_root) in cached_versions {
        if let Some(resolved) = resolve_from_cached_root(&crate_root, query)? {
            return Ok(Some(resolved));
        }
    }

    Ok(None)
}

fn resolve_from_cached_root(
    crate_root: &Path,
    query: &str,
) -> anyhow::Result<Option<ResolvedMetadataPath>> {
    if !crate_root.join("Cargo.toml").is_file() {
        return Ok(None);
    }

    resolve_source_from_metadata(crate_root, query)
}

fn cached_package_versions(package_root: &Path) -> anyhow::Result<Vec<(Version, PathBuf)>> {
    Ok(fs::read_dir(package_root)
        .with_context(|| format!("failed to read cache dir {}", package_root.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str()?;
            if file_name == "latest" {
                return None;
            }

            let crate_root = entry.path();
            if !crate_root.join("Cargo.toml").is_file() {
                return None;
            }

            Some((Version::parse(file_name).ok()?, crate_root))
        })
        .collect::<Vec<_>>())
}

fn clean_registry_cache(registry: Registry) -> anyhow::Result<()> {
    let cache_root = std::env::temp_dir()
        .join("gears-docs-cache")
        .join(sanitize_registry_name(registry));
    if cache_root.exists() {
        fs::remove_dir_all(&cache_root)
            .with_context(|| format!("failed to remove cache dir {}", cache_root.display()))?;
    }
    Ok(())
}

fn sanitize_registry_name(registry: Registry) -> String {
    registry
        .as_str()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn extract_crate_archive(
    archive_bytes: &[u8],
    package_root: &Path,
    crate_name: &str,
    version: &str,
) -> anyhow::Result<()> {
    let decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(package_root).with_context(|| {
        format!(
            "failed to unpack crate archive into {}",
            package_root.display()
        )
    })?;

    let extracted_root = package_root.join(format!("{crate_name}-{version}"));
    let crate_root = package_root.join(version);
    if extracted_root != crate_root && extracted_root.exists() && !crate_root.exists() {
        fs::rename(&extracted_root, &crate_root).with_context(|| {
            format!(
                "failed to move extracted crate from {} to {}",
                extracted_root.display(),
                crate_root.display()
            )
        })?;
    }

    if crate_root.join("Cargo.toml").is_file() {
        Ok(())
    } else {
        bail!("crate archive did not extract expected root for {crate_name} {version}")
    }
}

fn update_latest_symlink(package_root: &Path, version: &str) -> anyhow::Result<()> {
    let latest_link = package_root.join("latest");
    let target = Path::new(version);

    if let Ok(metadata) = fs::symlink_metadata(&latest_link) {
        if metadata.file_type().is_symlink() {
            remove_symlink(&latest_link)?;
        } else if metadata.is_dir() {
            fs::remove_dir_all(&latest_link).with_context(|| {
                format!(
                    "failed to remove existing latest entry {}",
                    latest_link.display()
                )
            })?;
        } else {
            fs::remove_file(&latest_link).with_context(|| {
                format!(
                    "failed to remove existing latest entry {}",
                    latest_link.display()
                )
            })?;
        }
    }

    create_dir_symlink(target, &latest_link)
}

#[cfg(unix)]
fn create_dir_symlink(target: &Path, link: &Path) -> anyhow::Result<()> {
    std::os::unix::fs::symlink(target, link).with_context(|| {
        format!(
            "failed to create symlink from {} to {}",
            link.display(),
            target.display()
        )
    })
}

#[cfg(windows)]
fn create_dir_symlink(target: &Path, link: &Path) -> anyhow::Result<()> {
    std::os::windows::fs::symlink_dir(target, link).with_context(|| {
        format!(
            "failed to create symlink from {} to {}",
            link.display(),
            target.display()
        )
    })
}

#[cfg(unix)]
fn remove_symlink(path: &Path) -> anyhow::Result<()> {
    fs::remove_file(path).with_context(|| format!("failed to remove symlink {}", path.display()))
}

#[cfg(windows)]
fn remove_symlink(path: &Path) -> anyhow::Result<()> {
    fs::remove_dir(path).with_context(|| format!("failed to remove symlink {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{
        Resolver, build_registry_client, find_dependency_spec, list_library_mappings,
        next_reexport_step, parse_dependencies, resolve_query_recursive,
        should_retry_registry_request,
    };
    use crate::common::Registry;
    use crate::gears_parser::resolve_source_from_metadata;
    use crate::gears_parser::test_utils::TempDirExt;
    use reqwest::{Method, StatusCode};
    use std::collections::HashSet;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn next_reexport_step_prefers_current_module_for_bare_reexports() {
        let project = TempDir::new().expect("temp dir should be created");
        project.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit"
            version = "0.5.4"
            edition = "2024"
            "#,
        );
        project.write(
            "src/lib.rs",
            r"
            pub mod gts;
            ",
        );
        project.write(
            "src/gts/mod.rs",
            r"
            pub mod plugin;
            pub use plugin::BaseGearsPluginV1;
            ",
        );
        project.write(
            "src/gts/plugin.rs",
            r"
            pub struct BaseGearsPluginV1;
            ",
        );

        let query = "cf-gears-toolkit::gts::BaseGearsPluginV1";
        let resolved = resolve_source_from_metadata(project.path(), query)
            .expect("metadata query should run")
            .expect("query should resolve");

        let next_step = next_reexport_step(project.path(), &resolved, query)
            .expect("re-export step should resolve")
            .expect("re-export step should exist");

        assert_eq!(
            next_step.query,
            "cf-gears-toolkit::gts::plugin::BaseGearsPluginV1"
        );
        assert!(next_step.requested_version.is_none());
        assert_eq!(next_step.preferred_path, project.path());
    }

    #[test]
    fn resolve_query_recursive_follows_bare_relative_reexports() {
        let project = TempDir::new().expect("temp dir should be created");
        project.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit"
            version = "0.5.4"
            edition = "2024"
            "#,
        );
        project.write(
            "src/lib.rs",
            r"
            pub mod gts;
            ",
        );
        project.write(
            "src/gts/mod.rs",
            r"
            pub mod plugin;
            pub use plugin::BaseGearsPluginV1;
            ",
        );
        project.write(
            "src/gts/plugin.rs",
            r"
            pub struct BaseGearsPluginV1;
            ",
        );

        let client = build_registry_client().expect("client should build");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime should build");
        let resolver = Resolver {
            workspace_path: project.path(),
            client: &client,
            runtime: &runtime,
            registry: Registry::CratesIo,
        };
        let mut visited = HashSet::new();

        let resolved_query = resolve_query_recursive(
            &resolver,
            project.path(),
            "cf-gears-toolkit::gts::BaseGearsPluginV1",
            None,
            &mut visited,
        )
        .expect("recursive resolution should succeed");

        assert!(
            resolved_query
                .source
                .contains("pub struct BaseGearsPluginV1;")
        );
        assert!(
            resolved_query
                .source_path
                .ends_with(Path::new("src/gts/plugin.rs"))
        );
    }

    #[test]
    fn parse_dependencies_inherits_workspace_package_name() {
        let project = TempDir::new().expect("temp dir should be created");
        project.write(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["cf-gears-toolkit", "cf-gears-toolkit-macros"]
            resolver = "3"

            [workspace.dependencies]
            gears_toolkit_macros = { package = "cf-gears-toolkit-macros", version = "0.5.4" }
            "#,
        );
        project.write(
            "cf-gears-toolkit/Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit"
            version = "0.5.4"
            edition = "2024"

            [dependencies]
            gears_toolkit_macros = { workspace = true }
            "#,
        );

        let dependencies = parse_dependencies(&project.path().join("cf-gears-toolkit"))
            .expect("dependencies parse");
        let dep = dependencies
            .get("gears_toolkit_macros")
            .expect("workspace dependency should be present");

        assert_eq!(dep.package_name, "cf-gears-toolkit-macros");
        assert_eq!(
            dep.version.as_ref().map(ToString::to_string).as_deref(),
            Some("0.5.4")
        );
    }

    #[test]
    fn find_dependency_spec_matches_hyphenated_manifest_keys() {
        let project = TempDir::new().expect("temp dir should be created");
        project.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit"
            version = "0.5.4"
            edition = "2024"

            [dependencies]
            cf-gears-toolkit-macros = "0.5.4"
            "#,
        );

        let dependencies = parse_dependencies(project.path()).expect("dependencies parse");
        let dep = find_dependency_spec(&dependencies, "cf_gears_toolkit_macros")
            .expect("hyphenated dependency should match underscore crate name");

        assert_eq!(dep.package_name, "cf-gears-toolkit-macros");
    }

    #[test]
    fn registry_retry_classifier_only_retries_transient_gets() {
        assert!(should_retry_registry_request(
            &Method::GET,
            Some(StatusCode::TOO_MANY_REQUESTS),
            false,
        ));
        assert!(should_retry_registry_request(
            &Method::GET,
            Some(StatusCode::SERVICE_UNAVAILABLE),
            false,
        ));
        assert!(should_retry_registry_request(&Method::GET, None, true));
        assert!(!should_retry_registry_request(
            &Method::GET,
            Some(StatusCode::NOT_FOUND),
            false,
        ));
        assert!(!should_retry_registry_request(
            &Method::POST,
            Some(StatusCode::SERVICE_UNAVAILABLE),
            false,
        ));
    }

    #[test]
    fn resolve_query_recursive_uses_workspace_package_name_for_external_reexports() {
        let project = TempDir::new().expect("temp dir should be created");
        project.write(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["cf-gears-toolkit", "cf-gears-toolkit-macros"]
            resolver = "3"

            [workspace.dependencies]
            gears_toolkit_macros = { package = "cf-gears-toolkit-macros", path = "cf-gears-toolkit-macros" }
            "#,
        );
        project.write(
            "cf-gears-toolkit/Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit"
            version = "0.5.4"
            edition = "2024"

            [dependencies]
            gears_toolkit_macros = { workspace = true }
            "#,
        );
        project.write(
            "cf-gears-toolkit/src/lib.rs",
            r"
            pub use gears_toolkit_macros::module;
            ",
        );
        project.write(
            "cf-gears-toolkit-macros/Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit-macros"
            version = "0.5.4"
            edition = "2024"
            "#,
        );
        project.write(
            "cf-gears-toolkit-macros/src/lib.rs",
            r"
            pub fn module() {}
            ",
        );

        let client = build_registry_client().expect("client should build");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime should build");
        let resolver = Resolver {
            workspace_path: project.path(),
            client: &client,
            runtime: &runtime,
            registry: Registry::CratesIo,
        };
        let mut visited = HashSet::new();

        let resolved_query = resolve_query_recursive(
            &resolver,
            &project.path().join("cf-gears-toolkit"),
            "cf-gears-toolkit::module",
            None,
            &mut visited,
        )
        .expect("recursive resolution should succeed");

        assert_eq!(resolved_query.package_name, "cf-gears-toolkit-macros");
        assert!(resolved_query.source.contains("pub fn module() {}"));
        assert!(
            resolved_query
                .source_path
                .ends_with(Path::new("cf-gears-toolkit-macros/src/lib.rs"))
        );
    }

    #[test]
    fn list_library_mappings_requires_package_only_query() {
        let project = TempDir::new().expect("temp dir should be created");
        project.write(
            "Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit"
            version = "0.5.4"
            edition = "2024"
            "#,
        );

        let error = list_library_mappings(Some(project.path()), "cf-gears-toolkit::module")
            .expect_err("non-package query should fail");

        assert!(
            error
                .to_string()
                .contains("--libs requires a package-only query")
        );
    }

    #[test]
    fn list_library_mappings_returns_source_code_names() {
        let project = TempDir::new().expect("temp dir should be created");
        project.write(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["cf-gears-toolkit", "cf-gears-toolkit-macros"]
            resolver = "3"
            "#,
        );
        project.write(
            "cf-gears-toolkit/Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit"
            version = "0.5.4"
            edition = "2024"

            [lib]
            name = "gears_toolkit"
            path = "src/lib.rs"

            [dependencies]
            gears_toolkit_macros = { package = "cf-gears-toolkit-macros", path = "../cf-gears-toolkit-macros" }
            "#,
        );
        project.write(
            "cf-gears-toolkit/src/lib.rs",
            r"
            pub use gears_toolkit_macros::module;
            ",
        );
        project.write(
            "cf-gears-toolkit-macros/Cargo.toml",
            r#"
            [package]
            name = "cf-gears-toolkit-macros"
            version = "0.5.4"
            edition = "2024"

            [lib]
            proc-macro = true
            "#,
        );
        project.write(
            "cf-gears-toolkit-macros/src/lib.rs",
            r"
            use proc_macro::TokenStream;

            #[proc_macro_attribute]
            pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
                item
            }
            ",
        );

        let mappings = list_library_mappings(
            Some(&project.path().join("cf-gears-toolkit")),
            "cf-gears-toolkit",
        )
        .expect("mappings should resolve");

        assert_eq!(
            mappings
                .iter()
                .map(|mapping| format!("{} -> {}", mapping.library_name, mapping.package_name))
                .collect::<Vec<_>>(),
            vec![
                "gears_toolkit -> cf-gears-toolkit".to_owned(),
                "gears_toolkit_macros -> cf-gears-toolkit-macros".to_owned(),
            ]
        );
    }
}
