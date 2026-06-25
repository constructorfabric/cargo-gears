use crate::gears_parser::{CargoTomlDependencies, CargoTomlDependency};
use anyhow::{Context, bail};
use cargo_generate::{GenerateArgs, TemplatePath, generate};
use semver::{Comparator, Op, Version, VersionReq};
use std::fs;
use std::path::{Path, PathBuf};

use super::{DEFAULT_BRANCH, DEFAULT_GIT_URL};

#[derive(Debug, Eq, PartialEq)]
pub struct GearParams {
    /// Template name (e.g. "background-worker").
    pub template: String,
    /// Gear name; defaults to the template name when absent.
    pub name: Option<String>,
    /// Path to the workspace root (defaults to current directory).
    pub path: PathBuf,
    /// Verbose output.
    pub verbose: bool,
    /// Path to a local template directory.
    pub local_path: Option<String>,
    /// URL to the git repo.
    pub git: Option<String>,
    /// Subfolder relative to the git repo.
    pub subfolder: Option<String>,
    /// Branch of the git repo.
    pub branch: Option<String>,
}

struct StagedModuleWrite {
    cargo_toml_path: PathBuf,
    doc: toml_edit::DocumentMut,
}

impl GearParams {
    pub fn run(&self) -> anyhow::Result<()> {
        ensure_gears_directory(&self.path)?;

        let generated_gears = self.generate_gear()?;
        println!("Gears {generated_gears:?} created");

        let (dependencies, staged_writes) = prepare_generated_gears(&self.path, &generated_gears)?;
        update_workspace_cargo_toml(&self.path, &generated_gears, dependencies)?;

        for staged in &staged_writes {
            save_toml_document(&staged.cargo_toml_path, &staged.doc)?;
        }

        Ok(())
    }

    /// Effective gear name: explicit `--name` or the template name.
    fn gear_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.template)
    }

    fn generate_gear(&self) -> anyhow::Result<Vec<String>> {
        let gear_name = self.gear_name();
        let gears_path = self.path.join("gears");
        let gear_path = gears_path.join(gear_name);
        if gear_path.exists() {
            bail!("gear {gear_name} already exists");
        }

        let resolved = self.resolve_template();

        generate(GenerateArgs {
            template_path: resolved,
            destination: Some(gears_path),
            name: Some(gear_name.to_string()),
            quiet: !self.verbose,
            verbose: self.verbose,
            no_workspace: true,
            ..GenerateArgs::default()
        })
        .with_context(|| format!("can't generate gear '{gear_name}'"))?;

        let mut generated = vec![format!("gears/{gear_name}")];

        let sdk_template = gear_path.join("sdk");
        if sdk_template.exists() {
            generated.push(format!("gears/{gear_name}/sdk"));
        }

        Ok(generated)
    }

    fn resolve_template(&self) -> TemplatePath {
        if let Some(local) = &self.local_path {
            return TemplatePath {
                path: Some(local.clone()),
                auto_path: self.subfolder.clone(),
                ..TemplatePath::default()
            };
        }

        let subfolder = self
            .subfolder
            .clone()
            .unwrap_or_else(|| format!("Modules/{}", self.template));

        TemplatePath {
            git: Some(self.git.as_deref().unwrap_or(DEFAULT_GIT_URL).to_owned()),
            branch: Some(self.branch.as_deref().unwrap_or(DEFAULT_BRANCH).to_owned()),
            auto_path: Some(subfolder),
            ..TemplatePath::default()
        }
    }
}

fn ensure_gears_directory(workspace_root: &Path) -> anyhow::Result<()> {
    let gears_dir = workspace_root.join("gears");
    if gears_dir.exists() {
        return Ok(());
    }

    bail!(
        "gears directory does not exist at {}. Make sure you are in a workspace initialized with 'generate workspace' or 'new'.",
        gears_dir.display()
    );
}

fn prepare_generated_gears(
    workspace_root: &Path,
    generated_gears: &[String],
) -> anyhow::Result<(CargoTomlDependencies, Vec<StagedModuleWrite>)> {
    let mut dependencies = CargoTomlDependencies::new();
    let mut staged_writes = Vec::new();

    for gear in generated_gears {
        let module_path = workspace_root.join(gear);
        let mut doc = get_cargo_toml(&module_path)?;
        for (name, incoming) in get_dependencies(&doc, &module_path, workspace_root)? {
            let Some(existing) = dependencies.get_mut(&name) else {
                dependencies.insert(name, incoming);
                continue;
            };

            let old_ver = existing.version.as_deref();
            let new_ver = incoming.version.as_deref();
            if should_replace_with_newer_semver(old_ver, new_ver.unwrap_or("")) {
                eprintln!(
                    "warning: dependency '{name}' version conflict: replacing '{}' with '{}'",
                    old_ver.unwrap_or("(none)"),
                    new_ver.unwrap_or("(none)"),
                );
                existing.version.clone_from(&incoming.version);
            } else if old_ver != new_ver {
                eprintln!(
                    "warning: dependency '{name}' version conflict: keeping '{}' over '{}'",
                    old_ver.unwrap_or("(none)"),
                    new_ver.unwrap_or("(none)"),
                );
            }
            merge_dependency_metadata(existing, &incoming);
        }

        let mut changed = rewrite_dependencies_to_workspace_inheritance(&mut doc);
        changed |= ensure_workspace_lints_inheritance(&mut doc);
        if changed {
            staged_writes.push(StagedModuleWrite {
                cargo_toml_path: module_path.join("Cargo.toml"),
                doc,
            });
        }
    }

    Ok((dependencies, staged_writes))
}

fn merge_dependency_metadata(existing: &mut CargoTomlDependency, incoming: &CargoTomlDependency) {
    match (existing.default_features, incoming.default_features) {
        (Some(e), Some(i)) => existing.default_features = Some(e && i),
        (None, Some(i)) => existing.default_features = Some(i),
        _ => {}
    }

    existing.features.extend(incoming.features.iter().cloned());

    if existing.path.is_none() && incoming.path.is_some() {
        existing.path.clone_from(&incoming.path);
    }

    if existing.package.is_none() && incoming.package.is_some() {
        existing.package.clone_from(&incoming.package);
    }
}

fn update_workspace_cargo_toml(
    workspace_root: &Path,
    generated_gears: &[String],
    dependencies: CargoTomlDependencies,
) -> anyhow::Result<()> {
    let mut workspace_doc = get_cargo_toml(workspace_root)?;
    add_members_to_workspace(&mut workspace_doc, generated_gears)?;
    add_dependencies_to_workspace(&mut workspace_doc, dependencies)?;
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    save_toml_document(&cargo_toml_path, &workspace_doc)
}

fn get_cargo_toml(path: &Path) -> anyhow::Result<toml_edit::DocumentMut> {
    let cargo_toml_path = path.join("Cargo.toml");
    fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("can't read {}", path.display()))?
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("can't parse {}", path.display()))
}

/// Extracts dependencies from the new modules' crate Cargo.toml
/// It sends to the root the package, version, path and default-features
/// The feature set are kept in the Cargo.toml of the module
fn get_dependencies(
    doc: &toml_edit::DocumentMut,
    source_crate_dir: &Path,
    workspace_root: &Path,
) -> anyhow::Result<CargoTomlDependencies> {
    let mut result = CargoTomlDependencies::new();
    let Some(dependencies) = doc["dependencies"].as_table() else {
        return Ok(result);
    };

    for (name, value) in dependencies {
        if value.as_str().is_none()
            && value.as_table().is_none()
            && value.as_inline_table().is_none()
        {
            eprintln!("warning: dependency '{name}' has unexpected TOML value type, skipping");
            continue;
        }

        let version = get_dep_str_field(value, "version");
        let raw_path = get_dep_str_field(value, "path");
        let package = get_dep_str_field(value, "package");
        let default_features = get_dep_bool_field(value, "default-features");

        let path = raw_path
            .map(|raw| workspace_relative_dependency_path(&raw, source_crate_dir, workspace_root))
            .transpose()?;

        let metadata = CargoTomlDependency {
            package: normalize_workspace_package_name(name, package.as_deref()),
            version,
            path,
            default_features,
            ..Default::default()
        };
        result.insert(name.to_string(), metadata);
    }

    Ok(result)
}

fn normalize_workspace_package_name(dep_name: &str, package: Option<&str>) -> Option<String> {
    package
        .map(str::trim)
        .filter(|package| !package.is_empty() && *package != dep_name)
        .map(ToOwned::to_owned)
}

fn workspace_relative_dependency_path(
    raw_path: &str,
    source_crate_dir: &Path,
    workspace_root: &Path,
) -> anyhow::Result<String> {
    let raw = Path::new(raw_path);
    let joined = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        source_crate_dir.join(raw)
    };

    let absolute = std::path::absolute(&joined)
        .with_context(|| format!("can't resolve absolute path for '{}'", joined.display()))?;
    let workspace_absolute = std::path::absolute(workspace_root).with_context(|| {
        format!(
            "can't resolve absolute path for '{}'",
            workspace_root.display()
        )
    })?;

    let relative = absolute
        .strip_prefix(&workspace_absolute)
        .with_context(|| {
            format!(
                "incorrect path for dependency '{}': not inside workspace '{}'",
                absolute.display(),
                workspace_absolute.display()
            )
        })?;

    let relative_str = relative.to_string_lossy().replace('\\', "/");
    if !relative_str.starts_with("gears/") {
        bail!(
            "incorrect path for dependency '{raw_path}': expected path under 'gears/', got '{relative_str}'",
        );
    }

    Ok(relative_str)
}

fn add_members_to_workspace(
    doc: &mut toml_edit::DocumentMut,
    gears: &[String],
) -> anyhow::Result<()> {
    let members = doc["workspace"]["members"]
        .as_array_mut()
        .context("workspace.members is not an array")?;
    for m in gears {
        let s = m.as_str();
        if !members
            .iter()
            .any(|x| matches!(x.as_str(), Some(inner) if inner == s))
        {
            members.push(m.clone());
        }
    }
    Ok(())
}

fn add_dependencies_to_workspace(
    doc: &mut toml_edit::DocumentMut,
    dependencies: CargoTomlDependencies,
) -> anyhow::Result<()> {
    let workspace_deps = doc["workspace"]["dependencies"]
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("workspace.dependencies is not a table")?;

    for (name, metadata) in dependencies {
        if let Some(existing_dep) = workspace_deps.get_mut(&name) {
            maybe_upgrade_workspace_dep_version(existing_dep, metadata.version.as_deref());
            maybe_apply_workspace_dep_default_features(existing_dep, metadata.default_features);
            maybe_apply_workspace_dep_source_key(
                existing_dep,
                &name,
                "package",
                metadata.package.as_deref(),
            )?;
            maybe_apply_workspace_dep_source_key(
                existing_dep,
                &name,
                "path",
                metadata.path.as_deref(),
            )?;
            continue;
        }
        workspace_deps.insert(
            &name,
            toml_edit::Item::Value(build_workspace_dep_inline_table(&name, metadata).into()),
        );
    }

    Ok(())
}

fn build_workspace_dep_inline_table(
    dependency_name: &str,
    metadata: CargoTomlDependency,
) -> toml_edit::InlineTable {
    let mut dep_table = toml_edit::InlineTable::new();

    if let Some(package) = metadata.package {
        dep_table.insert("package", package.into());
    }

    if let Some(default_features) = metadata.default_features {
        dep_table.insert("default-features", default_features.into());
    }

    if let Some(path) = metadata.path {
        dep_table.insert("path", path.into());
    } else if let Some(version) = metadata.version {
        dep_table.insert("version", version.into());
    } else {
        eprintln!(
            "warning: no version specified for dependency '{dependency_name}', using wildcard '*'"
        );
        dep_table.insert("version", "*".into());
    }

    dep_table
}

fn maybe_upgrade_workspace_dep_version(existing_dep: &mut toml_edit::Item, incoming: Option<&str>) {
    let Some(incoming_version) = incoming.map(str::trim).filter(|v| !v.is_empty()) else {
        return;
    };

    let existing_version = get_dep_str_field(existing_dep, "version");
    if should_replace_with_newer_semver(existing_version.as_deref(), incoming_version) {
        set_dep_field(existing_dep, "version", incoming_version);
    }
}

fn maybe_apply_workspace_dep_default_features(
    existing_dep: &mut toml_edit::Item,
    incoming: Option<bool>,
) {
    let Some(default_features) = incoming else {
        return;
    };

    set_dep_field(existing_dep, "default-features", default_features);
}

fn maybe_apply_workspace_dep_source_key(
    existing_dep: &mut toml_edit::Item,
    dep_name: &str,
    key: &str,
    incoming: Option<&str>,
) -> anyhow::Result<()> {
    let Some(incoming_value) = incoming.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(());
    };

    if let Some(existing_value) = get_dep_str_field(existing_dep, key) {
        if existing_value != incoming_value {
            bail!(
                "dependency '{dep_name}' has conflicting '{key}': existing '{existing_value}' vs incoming '{incoming_value}'",
            );
        }
        return Ok(());
    }

    set_dep_field(existing_dep, key, incoming_value);
    Ok(())
}

fn get_dep_value<'a>(dep: &'a toml_edit::Item, key: &str) -> Option<&'a toml_edit::Value> {
    dep.as_table()
        .and_then(|t| t.get(key))
        .and_then(toml_edit::Item::as_value)
        .or_else(|| dep.as_inline_table().and_then(|t| t.get(key)))
}

fn get_dep_str_field(dep: &toml_edit::Item, key: &str) -> Option<String> {
    if key == "version"
        && let Some(s) = dep.as_str()
    {
        return Some(s.to_owned());
    }
    get_dep_value(dep, key)
        .and_then(toml_edit::Value::as_str)
        .map(ToOwned::to_owned)
}

fn get_dep_bool_field(dep: &toml_edit::Item, key: &str) -> Option<bool> {
    get_dep_value(dep, key).and_then(toml_edit::Value::as_bool)
}

fn set_dep_field(dep: &mut toml_edit::Item, key: &str, value: impl Into<toml_edit::Value>) {
    let value = value.into();
    if let Some(table) = dep.as_table_mut() {
        table.insert(key, toml_edit::Item::Value(value));
        return;
    }
    if let Some(inline) = dep.as_inline_table_mut() {
        inline.insert(key, value);
        return;
    }
    promote_dep_to_inline_table(dep);
    if let Some(inline) = dep.as_inline_table_mut() {
        inline.insert(key, value);
    }
}

fn promote_dep_to_inline_table(dep: &mut toml_edit::Item) {
    let version = dep.as_str().unwrap_or("*").to_owned();
    let mut t = toml_edit::InlineTable::new();
    t.insert("version", version.into());
    *dep = toml_edit::Item::Value(t.into());
}

fn should_replace_with_newer_semver(existing: Option<&str>, incoming: &str) -> bool {
    let incoming = incoming.trim();
    if incoming.is_empty() || incoming == "*" {
        return false;
    }

    let Some(existing) = existing.map(str::trim).filter(|v| !v.is_empty()) else {
        return true;
    };

    if existing == "*" {
        return true;
    }

    match (version_req_floor(existing), version_req_floor(incoming)) {
        (Some(existing_floor), Some(incoming_floor)) => incoming_floor > existing_floor,
        _ => false,
    }
}

fn version_req_floor(req: &str) -> Option<Version> {
    let parsed = VersionReq::parse(req).ok()?;

    parsed.comparators.iter().filter_map(comparator_floor).max()
}

fn comparator_floor(comparator: &Comparator) -> Option<Version> {
    match comparator.op {
        Op::Exact | Op::Greater | Op::GreaterEq | Op::Tilde | Op::Caret => {
            let mut version = Version::new(
                comparator.major,
                comparator.minor.unwrap_or(0),
                comparator.patch.unwrap_or(0),
            );
            version.pre = comparator.pre.clone();
            Some(version)
        }
        _ => None,
    }
}

fn save_toml_document(path: &Path, doc: &toml_edit::DocumentMut) -> anyhow::Result<()> {
    let mut serialized = doc.to_string();
    if !serialized.ends_with('\n') {
        serialized.push('\n');
    }

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, serialized)
        .with_context(|| format!("can't write temp Cargo.toml file {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| format!("can't replace {}", path.display()))
}

fn rewrite_dependencies_to_workspace_inheritance(doc: &mut toml_edit::DocumentMut) -> bool {
    let Some(dependencies) = doc["dependencies"].as_table_mut() else {
        return false;
    };

    let mut changed = false;
    for (_, dependency) in dependencies.iter_mut() {
        changed |= rewrite_dependency_to_workspace_inheritance(dependency);
    }
    changed
}

fn ensure_workspace_lints_inheritance(doc: &mut toml_edit::DocumentMut) -> bool {
    let root = doc.as_table_mut();

    let Some(lints_item) = root.get_mut("lints") else {
        let mut lints_table = toml_edit::Table::new();
        lints_table.insert("workspace", toml_edit::value(true));
        root.insert("lints", toml_edit::Item::Table(lints_table));
        return true;
    };

    if get_dep_bool_field(lints_item, "workspace").is_some_and(|v| v) {
        return false;
    }

    set_dep_field(lints_item, "workspace", true);
    true
}

const WORKSPACE_INHERITED_KEYS: &[&str] = &[
    "version",
    "path",
    "default-features",
    "git",
    "branch",
    "tag",
    "rev",
];

fn rewrite_dependency_to_workspace_inheritance(dep: &mut toml_edit::Item) -> bool {
    if dep.as_str().is_some() {
        let mut dep_table = toml_edit::InlineTable::new();
        dep_table.insert("workspace", true.into());
        *dep = toml_edit::Item::Value(dep_table.into());
        return true;
    }

    let mut changed = false;
    for key in WORKSPACE_INHERITED_KEYS {
        changed |= remove_dep_key(dep, key);
    }
    if get_dep_bool_field(dep, "workspace").is_some_and(|v| v) {
        return changed;
    }
    set_dep_field(dep, "workspace", true);
    true
}

fn remove_dep_key(dep: &mut toml_edit::Item, key: &str) -> bool {
    if let Some(table) = dep.as_table_mut() {
        return table.remove(key).is_some();
    }
    if let Some(inline) = dep.as_inline_table_mut() {
        return inline.remove(key).is_some();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        add_dependencies_to_workspace, ensure_workspace_lints_inheritance, get_dependencies,
        normalize_workspace_package_name, rewrite_dependencies_to_workspace_inheritance,
        should_replace_with_newer_semver,
    };
    use crate::gears_parser::{CargoTomlDependencies, CargoTomlDependency};
    use std::collections::BTreeSet;

    #[test]
    fn replaces_workspace_dep_version_with_newer_semver() {
        let mut doc = r#"
            [workspace]
            [workspace.dependencies]
            reqwest = { version = "0.12", features = ["json"] }
        "#
        .parse::<toml_edit::DocumentMut>()
        .expect("workspace cargo toml");

        let mut dependencies = CargoTomlDependencies::new();
        dependencies.insert(
            "reqwest".to_owned(),
            CargoTomlDependency {
                version: Some("0.13".to_owned()),
                features: BTreeSet::from(["stream".to_owned()]),
                ..CargoTomlDependency::default()
            },
        );

        add_dependencies_to_workspace(&mut doc, dependencies).expect("add dependencies");

        let version = doc["workspace"]["dependencies"]["reqwest"]["version"]
            .as_str()
            .expect("reqwest version");
        assert_eq!(version, "0.13");
        let features = doc["workspace"]["dependencies"]["reqwest"]["features"]
            .as_array()
            .expect("reqwest features");
        assert_eq!(features.len(), 1);
        assert_eq!(
            features.get(0).and_then(toml_edit::Value::as_str),
            Some("json")
        );
    }

    #[test]
    fn forwards_default_features_but_not_features_for_new_workspace_dep() {
        let mut doc = r"
            [workspace]
            [workspace.dependencies]
        "
        .parse::<toml_edit::DocumentMut>()
        .expect("workspace cargo toml");

        let mut dependencies = CargoTomlDependencies::new();
        dependencies.insert(
            "reqwest".to_owned(),
            CargoTomlDependency {
                version: Some("0.13".to_owned()),
                default_features: Some(false),
                features: BTreeSet::from(["json".to_owned()]),
                ..CargoTomlDependency::default()
            },
        );

        add_dependencies_to_workspace(&mut doc, dependencies).expect("add dependencies");

        let reqwest_dep = doc["workspace"]["dependencies"]["reqwest"]
            .as_inline_table()
            .expect("reqwest should be inline table");
        assert_eq!(
            reqwest_dep
                .get("version")
                .and_then(toml_edit::Value::as_str),
            Some("0.13")
        );
        assert_eq!(
            reqwest_dep
                .get("default-features")
                .and_then(toml_edit::Value::as_bool),
            Some(false)
        );
        assert!(reqwest_dep.get("features").is_none());
    }

    #[test]
    fn keeps_workspace_dep_version_when_incoming_is_older() {
        let mut doc = r#"
            [workspace]
            [workspace.dependencies]
            reqwest = { version = "0.13" }
        "#
        .parse::<toml_edit::DocumentMut>()
        .expect("workspace cargo toml");

        let mut dependencies = CargoTomlDependencies::new();
        dependencies.insert(
            "reqwest".to_owned(),
            CargoTomlDependency {
                version: Some("0.12".to_owned()),
                ..CargoTomlDependency::default()
            },
        );

        add_dependencies_to_workspace(&mut doc, dependencies).expect("add dependencies");

        let version = doc["workspace"]["dependencies"]["reqwest"]["version"]
            .as_str()
            .expect("reqwest version");
        assert_eq!(version, "0.13");
    }

    #[test]
    fn rewrites_module_deps_to_workspace_and_keeps_feature_flags() {
        let mut doc = r#"
            [dependencies]
            serde = { version = "1.0", path = "../deps/serde", features = ["derive"], default-features = false }
            anyhow = "1.0"
        "#
            .parse::<toml_edit::DocumentMut>()
            .expect("module cargo toml");

        let changed = rewrite_dependencies_to_workspace_inheritance(&mut doc);
        assert!(changed);

        let serde_dep = &doc["dependencies"]["serde"];
        let serde_inline = serde_dep
            .as_inline_table()
            .expect("serde should be an inline table");
        assert_eq!(
            serde_inline
                .get("workspace")
                .and_then(toml_edit::Value::as_bool),
            Some(true)
        );
        assert!(serde_inline.get("version").is_none());
        assert!(serde_inline.get("path").is_none());
        assert_eq!(
            serde_inline
                .get("default-features")
                .and_then(toml_edit::Value::as_bool),
            None
        );
        let features = serde_inline
            .get("features")
            .and_then(toml_edit::Value::as_array)
            .expect("serde features");
        assert_eq!(features.len(), 1);
        assert_eq!(
            features.get(0).and_then(toml_edit::Value::as_str),
            Some("derive")
        );

        let anyhow_dep = &doc["dependencies"]["anyhow"];
        let anyhow_inline = anyhow_dep
            .as_inline_table()
            .expect("anyhow should be an inline table");
        assert_eq!(
            anyhow_inline
                .get("workspace")
                .and_then(toml_edit::Value::as_bool),
            Some(true)
        );
        assert!(anyhow_inline.get("version").is_none());
    }

    #[test]
    fn newer_semver_detection_handles_missing_and_wildcard_versions() {
        assert!(should_replace_with_newer_semver(None, "1.2.3"));
        assert!(should_replace_with_newer_semver(Some("*"), "1.2.3"));
        assert!(!should_replace_with_newer_semver(Some("1.2.3"), "*"));
        assert!(!should_replace_with_newer_semver(Some("1.2.3"), "1.2.2"));
    }

    #[test]
    fn collects_dependencies_with_workspace_relative_paths() {
        let doc = r#"
            [dependencies]
            module_sdk = { path = "./sdk", version = "0.1.0" }
        "#
        .parse::<toml_edit::DocumentMut>()
        .expect("module cargo toml");

        let workspace_root = std::env::current_dir().expect("current_dir");
        let gear_dir = workspace_root.join("gears").join("api-gateway");

        let dependencies =
            get_dependencies(&doc, &gear_dir, &workspace_root).expect("get_dependencies");

        let sdk_dep = dependencies
            .get("module_sdk")
            .expect("module_sdk dependency");
        assert_eq!(sdk_dep.path.as_deref(), Some("gears/api-gateway/sdk"));
    }

    #[test]
    fn forwards_package_only_for_renamed_dependencies() {
        let doc = r#"
            [dependencies]
            serde = { package = "serde", version = "1.0", default-features = false }
            serde_json_alias = { package = "serde_json", version = "1.0" }
            anyhow = "1.0"
        "#
        .parse::<toml_edit::DocumentMut>()
        .expect("module cargo toml");

        let workspace_root = std::env::current_dir().expect("current_dir");
        let gear_dir = workspace_root.join("gears").join("api-gateway");

        let dependencies =
            get_dependencies(&doc, &gear_dir, &workspace_root).expect("get_dependencies");

        assert_eq!(
            dependencies
                .get("serde")
                .and_then(|dep| dep.package.as_deref()),
            None
        );
        assert_eq!(
            dependencies
                .get("serde")
                .and_then(|dep| dep.default_features),
            Some(false)
        );
        assert_eq!(
            dependencies
                .get("serde_json_alias")
                .and_then(|dep| dep.package.as_deref()),
            Some("serde_json")
        );
        assert_eq!(
            dependencies
                .get("anyhow")
                .and_then(|dep| dep.package.as_deref()),
            None
        );
    }

    #[test]
    fn normalize_workspace_package_name_omits_equal_names() {
        assert_eq!(
            normalize_workspace_package_name("serde", Some("serde")),
            None
        );
        assert_eq!(
            normalize_workspace_package_name("serde_alias", Some("serde")),
            Some("serde".to_owned())
        );
    }

    #[test]
    fn adds_lints_workspace_section_when_missing() {
        let mut doc = r#"
            [package]
            name = "demo"
            version = "0.1.0"
        "#
        .parse::<toml_edit::DocumentMut>()
        .expect("module cargo toml");

        let changed = ensure_workspace_lints_inheritance(&mut doc);
        assert!(changed);
        assert_eq!(doc["lints"]["workspace"].as_bool(), Some(true));
    }

    #[test]
    fn keeps_existing_workspace_lints_unchanged() {
        let mut doc = r"
            [lints]
            workspace = true
        "
        .parse::<toml_edit::DocumentMut>()
        .expect("module cargo toml");

        let changed = ensure_workspace_lints_inheritance(&mut doc);
        assert!(!changed);
        assert_eq!(doc["lints"]["workspace"].as_bool(), Some(true));
    }
}
