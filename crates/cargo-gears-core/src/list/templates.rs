use crate::manifest::{Manifest, TemplateDefinition, TemplateSource};
use anyhow::{Context, bail};
use cargo_generate::TemplatePath;
use serde::Serialize;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub const DEFAULT_GIT_URL: &str = "git@github.com:Bechma/cf-template-rust.git";
pub const DEFAULT_BRANCH: &str = "main";

const fn source_path(folder: &str) -> TemplateSource<'_> {
    TemplateSource::Git {
        url: Cow::Borrowed(DEFAULT_GIT_URL),
        revision: None,
        tag: None,
        branch: Some(Cow::Borrowed(DEFAULT_BRANCH)),
        subfolder: Some(Cow::Borrowed(folder)),
    }
}

pub const BUILTIN_TEMPLATES_GEARS: &[TemplateDefinition] = &[
    TemplateDefinition {
        name: Cow::Borrowed("pokemon-demo"),
        description: Cow::Borrowed(
            "pokemon demo that shows a rest api with infra capabilities(external requests) and a database",
        ),
        source: source_path("Modules/api-db-handler"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("api-fetcher"),
        description: Cow::Borrowed("simple gear example that only handles external requests"),
        source: source_path("Modules/api-fetcher"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("api-gateway"),
        description: Cow::Borrowed(
            "simple gateway required for the system to have rest api. Prefer to use the system gear cf-gears-api-gateway",
        ),
        source: source_path("Modules/api-gateway"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("background-worker"),
        description: Cow::Borrowed("simple gear example that spawn a background worker"),
        source: source_path("Modules/background-worker"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("db-handler"),
        description: Cow::Borrowed("simple gear example that only handles database interactions"),
        source: source_path("Modules/db-handler"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("rest-api"),
        description: Cow::Borrowed("simple gear example that only handles rest api calls"),
        source: source_path("Modules/rest-api"),
    },
];

pub const BUILTIN_TEMPLATES_WORKSPACE: &[TemplateDefinition] = &[TemplateDefinition {
    name: Cow::Borrowed("basic-init"),
    description: Cow::Borrowed("skeleton to initialize a new gears project"),
    source: source_path("Init"),
}];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateKind {
    Gear,
    Workspace,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TemplatesParams {
    pub path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct TemplateListing {
    templates: Vec<ListedTemplate>,
}

#[derive(Debug, Serialize)]
struct ListedTemplate {
    kind: &'static str,
    name: String,
    description: String,
    origin: &'static str,
}

impl TemplatesParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_path = crate::common::resolve_workspace_path(self.path.as_deref())?;
        print_templates(None, &workspace_path)
    }
}

pub fn print_templates(kind: Option<TemplateKind>, workspace_path: &Path) -> anyhow::Result<()> {
    let templates = collect_templates(kind, workspace_path)?;

    for (index, template) in templates.templates.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!("{} template: {}", template.kind, template.name);
        println!("description: {}", template.description);
        println!("origin: {}", template.origin);
    }

    Ok(())
}

pub fn resolve_template_path(
    kind: TemplateKind,
    name: &str,
    workspace_path: &Path,
) -> anyhow::Result<TemplatePath> {
    let lookup_name = workspace_template_alias(kind, name);
    let builtins = builtin_templates(kind);
    if let Some(template) = builtins
        .iter()
        .find(|template| template.name.as_ref() == lookup_name)
    {
        return template_source_to_path(&template.source, workspace_path);
    }

    if let Some(manifest) = load_manifest(workspace_path)?
        && let Some(template) = manifest_templates(&manifest, kind)
            .iter()
            .find(|template| template.name.as_ref() == lookup_name)
    {
        return template_source_to_path(&template.source, workspace_path);
    }

    bail!(
        "unknown {} template '{}'. Available templates: {}",
        kind.as_str(),
        name,
        available_template_names(kind, workspace_path)?.join(", ")
    );
}

fn collect_templates(
    kind: Option<TemplateKind>,
    workspace_path: &Path,
) -> anyhow::Result<TemplateListing> {
    let manifest = load_manifest(workspace_path)?;
    let mut templates = Vec::new();

    for template_kind in [TemplateKind::Workspace, TemplateKind::Gear] {
        if kind.is_some_and(|kind| kind != template_kind) {
            continue;
        }

        templates.extend(
            builtin_templates(template_kind)
                .iter()
                .map(|template| listed_template(template_kind, template, "builtin")),
        );

        if let Some(manifest) = &manifest {
            templates.extend(
                manifest_templates(manifest, template_kind)
                    .iter()
                    .map(|template| listed_template(template_kind, template, "manifest")),
            );
        }
    }

    Ok(TemplateListing { templates })
}

fn listed_template(
    kind: TemplateKind,
    template: &TemplateDefinition<'_>,
    origin: &'static str,
) -> ListedTemplate {
    ListedTemplate {
        kind: kind.as_str(),
        name: template.name.to_string(),
        description: template.description.to_string(),
        origin,
    }
}

fn available_template_names(
    kind: TemplateKind,
    workspace_path: &Path,
) -> anyhow::Result<Vec<String>> {
    Ok(collect_templates(Some(kind), workspace_path)?
        .templates
        .into_iter()
        .map(|template| template.name)
        .collect())
}

const fn builtin_templates(kind: TemplateKind) -> &'static [TemplateDefinition<'static>] {
    match kind {
        TemplateKind::Gear => BUILTIN_TEMPLATES_GEARS,
        TemplateKind::Workspace => BUILTIN_TEMPLATES_WORKSPACE,
    }
}

fn manifest_templates<'a>(
    manifest: &'a Manifest<'a>,
    kind: TemplateKind,
) -> &'a [TemplateDefinition<'a>] {
    let Some(templates) = &manifest.templates else {
        return &[];
    };

    match kind {
        TemplateKind::Gear => &templates.gear,
        TemplateKind::Workspace => &templates.workspace,
    }
}

fn load_manifest(workspace_path: &Path) -> anyhow::Result<Option<Manifest<'static>>> {
    let manifest_path = workspace_path.join(crate::manifest::DEFAULT_MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Ok(None);
    }

    Manifest::load(&manifest_path).map(Some)
}

fn template_source_to_path(
    source: &TemplateSource<'_>,
    workspace_path: &Path,
) -> anyhow::Result<TemplatePath> {
    let template_path: anyhow::Result<TemplatePath> = match source {
        TemplateSource::Git {
            url,
            revision,
            tag,
            branch,
            subfolder,
        } => Ok(TemplatePath {
            git: Some(url.to_string()),
            revision: revision.as_ref().map(ToString::to_string),
            tag: tag.as_ref().map(ToString::to_string),
            branch: branch.as_ref().map(ToString::to_string),
            auto_path: subfolder.as_ref().map(ToString::to_string),
            ..TemplatePath::default()
        }),
        TemplateSource::Local { path } => {
            let path = Path::new(path.as_ref());
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                workspace_path.join(path)
            };
            Ok(TemplatePath {
                path: Some(path.to_string_lossy().into_owned()),
                ..TemplatePath::default()
            })
        }
        TemplateSource::Embedded => {
            bail!("embedded templates are not supported for generate commands")
        }
    };

    template_path.context("failed to resolve template source")
}

fn workspace_template_alias(kind: TemplateKind, name: &str) -> &str {
    if kind == TemplateKind::Workspace && name == "default" {
        "basic-init"
    } else {
        name
    }
}

impl TemplateKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Gear => "gear",
            Self::Workspace => "workspace",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gears_parser::test_utils::TempDirExt;
    use tempfile::TempDir;

    #[test]
    fn workspace_default_alias_resolves_builtin_template() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        let template = resolve_template_path(TemplateKind::Workspace, "default", temp_dir.path())
            .expect("default workspace template should resolve");

        assert_eq!(template.git.as_deref(), Some(DEFAULT_GIT_URL));
        assert_eq!(template.branch.as_deref(), Some(DEFAULT_BRANCH));
        assert_eq!(template.auto_path.as_deref(), Some("Init"));
    }

    #[test]
    fn gear_template_resolves_from_manifest_registry() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        temp_dir.write(
            "Gears.toml",
            r#"
[apps.app.dev]
config = "app-dev.yml"
gears = []

[[templates.gear]]
name = "custom-gear"
description = "custom gear template"
source = { source = "local", path = "templates/custom-gear" }
"#,
        );

        let template = resolve_template_path(TemplateKind::Gear, "custom-gear", temp_dir.path())
            .expect("manifest gear template should resolve");

        assert_eq!(
            template.path.as_deref(),
            Some(
                temp_dir
                    .path()
                    .join("templates/custom-gear")
                    .to_string_lossy()
                    .as_ref()
            )
        );
    }
}
