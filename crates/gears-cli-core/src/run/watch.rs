use crate::manifest::WatchPolicy;
use crate::module_parser::CargoTomlDependencies;
use anyhow::Context;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::run) enum WatchAction {
    Restart,
    Regenerate,
}

#[derive(Clone, Copy)]
pub(in crate::run) struct WatchPlanInputs<'a> {
    pub workspace_path: &'a Path,
    pub manifest_path: &'a Path,
    pub config_path: &'a Path,
    pub dependencies: &'a CargoTomlDependencies,
}

pub(in crate::run) struct WatchPlan {
    workspace_path: PathBuf,
    targets: Vec<WatchTarget>,
    excluded_paths: Vec<PathBuf>,
}

impl WatchPlan {
    pub(in crate::run) fn from_policy(
        policy: &WatchPolicy,
        inputs: WatchPlanInputs<'_>,
    ) -> anyhow::Result<Self> {
        let workspace_path = canonicalize_watch_path(Path::new(""), inputs.workspace_path)?;
        let excluded_paths = policy
            .exclude
            .iter()
            .map(|path| canonicalize_watch_path(&workspace_path, path))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let targets = if policy.include.is_empty() {
            default_targets(inputs, &workspace_path)?
        } else {
            policy
                .include
                .iter()
                .map(|path| {
                    let path = canonicalize_watch_path(&workspace_path, path)?;
                    let action = default_action_for_path(&path, inputs, &workspace_path)
                        .context("failed to resolve default watch action")?
                        .unwrap_or(WatchAction::Restart);
                    Ok(WatchTarget::infer(path, action))
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        }
        .into_iter()
        .filter(|target| !is_excluded(target.path(), &excluded_paths))
        .collect();

        Ok(Self {
            workspace_path,
            targets,
            excluded_paths,
        })
    }

    pub(in crate::run) fn watch(&self, watcher: &mut impl Watcher) -> anyhow::Result<()> {
        let mut registered_roots = HashSet::new();
        for target in &self.targets {
            let root = target.watch_root()?;
            if registered_roots.insert((root.path.clone(), root.recursive)) {
                watcher
                    .watch(&root.path, root.mode())
                    .with_context(|| format!("failed to watch {}", root.path.display()))?;
            }
        }
        Ok(())
    }

    pub(in crate::run) fn action_for_event(
        &self,
        event: &Event,
    ) -> anyhow::Result<Option<WatchAction>> {
        if !is_relevant_event(event.kind) {
            return Ok(None);
        }

        self.action_for_paths(event.paths.iter().map(PathBuf::as_path))
    }

    fn action_for_paths<'a>(
        &self,
        paths: impl IntoIterator<Item = &'a Path>,
    ) -> anyhow::Result<Option<WatchAction>> {
        let action = paths
            .into_iter()
            .map(|path| canonicalize_watch_path(&self.workspace_path, path))
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .filter(|path| !is_excluded(path, &self.excluded_paths))
            .filter_map(|path| self.action_for_path(&path))
            .max();
        Ok(action)
    }

    fn action_for_path(&self, path: &Path) -> Option<WatchAction> {
        self.targets
            .iter()
            .filter_map(|target| target.action_for_path(path))
            .max()
    }
}

fn default_targets(
    inputs: WatchPlanInputs<'_>,
    workspace_path: &Path,
) -> anyhow::Result<Vec<WatchTarget>> {
    let mut targets = vec![
        WatchTarget::file(
            canonicalize_watch_path(workspace_path, Path::new("Cargo.toml"))?,
            WatchAction::Regenerate,
        ),
        WatchTarget::file(
            canonicalize_watch_path(workspace_path, inputs.manifest_path)?,
            WatchAction::Regenerate,
        ),
        WatchTarget::file(
            canonicalize_watch_path(workspace_path, inputs.config_path)?,
            WatchAction::Restart,
        ),
    ];

    targets.extend(
        dependency_paths(inputs.dependencies, workspace_path)?
            .into_iter()
            .map(|path| WatchTarget::directory(path, WatchAction::Restart)),
    );

    Ok(targets)
}

fn dependency_paths(
    deps: &CargoTomlDependencies,
    workspace_path: &Path,
) -> anyhow::Result<Vec<PathBuf>> {
    deps.values()
        .filter_map(|dependency| dependency.path.as_deref())
        .map(|path| canonicalize_watch_path(workspace_path, Path::new(path)))
        .collect()
}

fn default_action_for_path(
    path: &Path,
    inputs: WatchPlanInputs<'_>,
    workspace_path: &Path,
) -> anyhow::Result<Option<WatchAction>> {
    let cargo_manifest = canonicalize_watch_path(workspace_path, Path::new("Cargo.toml"))?;
    let manifest = canonicalize_watch_path(workspace_path, inputs.manifest_path)?;

    Ok((path == cargo_manifest || path == manifest).then_some(WatchAction::Regenerate))
}

fn canonicalize_watch_path(base_path: &Path, path: &Path) -> anyhow::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_path.join(path)
    };

    absolute
        .canonicalize()
        .with_context(|| format!("can't canonicalize watch path {}", absolute.display()))
}

const fn is_relevant_event(kind: EventKind) -> bool {
    matches!(
        kind,
        EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn is_excluded(path: &Path, excluded_paths: &[PathBuf]) -> bool {
    debug_assert!(path.is_absolute(), "watch path must be absolute");
    debug_assert!(
        excluded_paths.iter().all(|path| path.is_absolute()),
        "excluded watch paths must be absolute"
    );

    excluded_paths
        .iter()
        .any(|excluded| path.starts_with(excluded))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatchTargetKind {
    File,
    Directory,
}

#[derive(Debug, Clone)]
struct WatchTarget {
    path: PathBuf,
    kind: WatchTargetKind,
    action: WatchAction,
}

impl WatchTarget {
    fn infer(path: PathBuf, action: WatchAction) -> Self {
        if path.is_dir() {
            Self::directory(path, action)
        } else {
            Self::file(path, action)
        }
    }

    const fn file(path: PathBuf, action: WatchAction) -> Self {
        Self {
            path,
            kind: WatchTargetKind::File,
            action,
        }
    }

    const fn directory(path: PathBuf, action: WatchAction) -> Self {
        Self {
            path,
            kind: WatchTargetKind::Directory,
            action,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn matches(&self, path: &Path) -> bool {
        match self.kind {
            WatchTargetKind::File => path == self.path,
            WatchTargetKind::Directory => path.starts_with(&self.path),
        }
    }

    fn action_for_path(&self, path: &Path) -> Option<WatchAction> {
        self.matches(path).then_some(self.action)
    }

    fn watch_root(&self) -> anyhow::Result<WatchRoot> {
        match self.kind {
            WatchTargetKind::File => {
                let parent = self
                    .path
                    .parent()
                    .with_context(|| format!("watch path {} has no parent", self.path.display()))?;
                Ok(WatchRoot {
                    path: parent.to_path_buf(),
                    recursive: false,
                })
            }
            WatchTargetKind::Directory => Ok(WatchRoot {
                path: self.path.clone(),
                recursive: true,
            }),
        }
    }
}

#[derive(Debug, Clone)]
struct WatchRoot {
    path: PathBuf,
    recursive: bool,
}

impl WatchRoot {
    const fn mode(&self) -> RecursiveMode {
        if self.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_parser::CargoTomlDependency;
    use std::fs;
    use tempfile::TempDir;

    fn inputs<'a>(
        workspace_path: &'a Path,
        manifest_path: &'a Path,
        config_path: &'a Path,
        dependencies: &'a CargoTomlDependencies,
    ) -> WatchPlanInputs<'a> {
        WatchPlanInputs {
            workspace_path,
            manifest_path,
            config_path,
            dependencies,
        }
    }

    fn assert_plan_paths_are_absolute(plan: &WatchPlan) {
        assert!(plan.workspace_path.is_absolute());
        assert!(
            plan.targets
                .iter()
                .all(|target| target.path().is_absolute())
        );
        assert!(plan.excluded_paths.iter().all(|path| path.is_absolute()));
    }

    #[test]
    fn default_watch_plan_tracks_manifest_metadata_and_local_modules() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        fs::create_dir_all(workspace.join("crates/local-module/src")).unwrap();
        fs::create_dir_all(workspace.join("config")).unwrap();
        fs::write(workspace.join("Cargo.toml"), "").unwrap();
        fs::write(workspace.join("Gears.toml"), "").unwrap();
        fs::write(workspace.join("config/app.yml"), "").unwrap();
        fs::write(workspace.join("crates/local-module/src/lib.rs"), "").unwrap();
        let manifest_path = workspace.join("Gears.toml");
        let config_path = workspace.join("config/app.yml");
        let mut dependencies = CargoTomlDependencies::new();
        dependencies.insert(
            "local_module".to_owned(),
            CargoTomlDependency {
                path: Some("crates/local-module".to_owned()),
                ..Default::default()
            },
        );

        let plan = WatchPlan::from_policy(
            &WatchPolicy::default(),
            inputs(workspace, &manifest_path, &config_path, &dependencies),
        )
        .unwrap();
        assert_plan_paths_are_absolute(&plan);

        let cargo_manifest = workspace.join("Cargo.toml");
        let gears_manifest = workspace.join("Gears.toml");
        assert_eq!(
            plan.action_for_paths([cargo_manifest.as_path()]).unwrap(),
            Some(WatchAction::Regenerate)
        );
        assert_eq!(
            plan.action_for_paths([gears_manifest.as_path()]).unwrap(),
            Some(WatchAction::Regenerate)
        );
        assert_eq!(
            plan.action_for_paths([Path::new("crates/local-module/src/lib.rs")])
                .unwrap(),
            Some(WatchAction::Restart)
        );
    }

    #[test]
    fn include_replaces_default_watch_targets() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("config")).unwrap();
        fs::write(workspace.join("Cargo.toml"), "").unwrap();
        fs::write(workspace.join("Gears.toml"), "").unwrap();
        fs::write(workspace.join("config/app.yml"), "").unwrap();
        fs::write(workspace.join("src/main.rs"), "").unwrap();
        let manifest_path = workspace.join("Gears.toml");
        let config_path = workspace.join("config/app.yml");
        let dependencies = CargoTomlDependencies::new();
        let policy = WatchPolicy {
            include: vec![PathBuf::from("src")],
            ..WatchPolicy::default()
        };

        let plan = WatchPlan::from_policy(
            &policy,
            inputs(workspace, &manifest_path, &config_path, &dependencies),
        )
        .unwrap();
        assert_plan_paths_are_absolute(&plan);

        let source_file = workspace.join("src/main.rs");
        assert_eq!(
            plan.action_for_paths([Path::new("Cargo.toml")]).unwrap(),
            None
        );
        assert_eq!(
            plan.action_for_paths([source_file.as_path()]).unwrap(),
            Some(WatchAction::Restart)
        );
    }

    #[test]
    fn regenerate_action_wins_when_a_path_matches_multiple_targets() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        fs::create_dir_all(workspace.join("config")).unwrap();
        fs::write(workspace.join("Cargo.toml"), "").unwrap();
        fs::write(workspace.join("Gears.toml"), "").unwrap();
        fs::write(workspace.join("config/app.yml"), "").unwrap();
        let manifest_path = workspace.join("Gears.toml");
        let config_path = workspace.join("config/app.yml");
        let dependencies = CargoTomlDependencies::new();
        let policy = WatchPolicy {
            include: vec![PathBuf::from("."), PathBuf::from("Cargo.toml")],
            ..WatchPolicy::default()
        };

        let plan = WatchPlan::from_policy(
            &policy,
            inputs(workspace, &manifest_path, &config_path, &dependencies),
        )
        .unwrap();
        assert_plan_paths_are_absolute(&plan);

        assert_eq!(
            plan.action_for_paths([Path::new("Cargo.toml")]).unwrap(),
            Some(WatchAction::Regenerate)
        );
    }

    #[test]
    fn missing_include_path_returns_canonicalization_error() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        fs::create_dir_all(workspace.join("config")).unwrap();
        fs::write(workspace.join("Cargo.toml"), "").unwrap();
        fs::write(workspace.join("Gears.toml"), "").unwrap();
        fs::write(workspace.join("config/app.yml"), "").unwrap();
        let manifest_path = workspace.join("Gears.toml");
        let config_path = workspace.join("config/app.yml");
        let dependencies = CargoTomlDependencies::new();
        let policy = WatchPolicy {
            include: vec![PathBuf::from("missing")],
            ..WatchPolicy::default()
        };

        let Err(err) = WatchPlan::from_policy(
            &policy,
            inputs(workspace, &manifest_path, &config_path, &dependencies),
        ) else {
            panic!("missing include path should fail")
        };
        assert!(err.to_string().contains("can't canonicalize watch path"));
    }

    #[test]
    fn missing_event_path_returns_canonicalization_error() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("config")).unwrap();
        fs::write(workspace.join("Cargo.toml"), "").unwrap();
        fs::write(workspace.join("Gears.toml"), "").unwrap();
        fs::write(workspace.join("config/app.yml"), "").unwrap();
        fs::write(workspace.join("src/main.rs"), "").unwrap();
        let manifest_path = workspace.join("Gears.toml");
        let config_path = workspace.join("config/app.yml");
        let dependencies = CargoTomlDependencies::new();
        let policy = WatchPolicy {
            include: vec![PathBuf::from("src")],
            ..WatchPolicy::default()
        };
        let plan = WatchPlan::from_policy(
            &policy,
            inputs(workspace, &manifest_path, &config_path, &dependencies),
        )
        .unwrap();

        let err = plan
            .action_for_paths([Path::new("src/missing.rs")])
            .unwrap_err();

        assert!(err.to_string().contains("can't canonicalize watch path"));
    }

    #[test]
    fn exclude_filters_default_watch_targets() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        fs::create_dir_all(workspace.join("crates/local-module/src")).unwrap();
        fs::create_dir_all(workspace.join("config")).unwrap();
        fs::write(workspace.join("Cargo.toml"), "").unwrap();
        fs::write(workspace.join("Gears.toml"), "").unwrap();
        fs::write(workspace.join("config/app.yml"), "").unwrap();
        fs::write(workspace.join("crates/local-module/src/lib.rs"), "").unwrap();
        let manifest_path = workspace.join("Gears.toml");
        let config_path = workspace.join("config/app.yml");
        let mut dependencies = CargoTomlDependencies::new();
        dependencies.insert(
            "local_module".to_owned(),
            CargoTomlDependency {
                path: Some("crates/local-module".to_owned()),
                ..Default::default()
            },
        );
        let policy = WatchPolicy {
            exclude: vec![PathBuf::from("crates/local-module")],
            ..WatchPolicy::default()
        };

        let plan = WatchPlan::from_policy(
            &policy,
            inputs(workspace, &manifest_path, &config_path, &dependencies),
        )
        .unwrap();
        assert_plan_paths_are_absolute(&plan);

        let cargo_manifest = workspace.join("Cargo.toml");
        assert_eq!(
            plan.action_for_paths([Path::new("crates/local-module/src/lib.rs")])
                .unwrap(),
            None
        );
        assert_eq!(
            plan.action_for_paths([cargo_manifest.as_path()]).unwrap(),
            Some(WatchAction::Regenerate)
        );
    }
}
