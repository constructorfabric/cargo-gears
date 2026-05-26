use anyhow::{Context, bail};
use cargo_generate::{GenerateArgs, TemplatePath, generate};
use std::path::PathBuf;

use super::{DEFAULT_BRANCH, DEFAULT_GIT_URL};

/// Content of SKILL.md embedded at compile time.
const SKILL_MD_CONTENT: &str = include_str!("../../../../SKILL.md");

/// Content of Dockerfile embedded at compile time.
const DOCKERFILE_CONTENT: &str = include_str!("../../shared/Dockerfile");

/// Content of .dockerignore embedded at compile time.
const DOCKERIGNORE_CONTENT: &str = include_str!("../../shared/.dockerignore");

#[derive(Debug, Eq, PartialEq)]
pub struct WorkspaceArgs {
    /// Path to initialize the project.
    pub path: PathBuf,
    /// Template name (defaults to "default").
    pub template: String,
    /// Name of the project; inferred from the path if not specified.
    pub name: Option<String>,
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
    /// Overwrite existing files.
    pub r#override: bool,
}

impl WorkspaceArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        if self.path.exists() && !self.path.is_dir() {
            bail!("path is not a directory");
        }
        if !self.path.exists() {
            std::fs::create_dir_all(&self.path).context("path can't be created")?;
        }

        let name = match &self.name {
            Some(name) => name.as_str(),
            None => self
                .path
                .file_name()
                .context("we can't infer the name from the path, use --name")?
                .to_str()
                .context("name is strange")?,
        };

        let resolved = self.resolve_template();

        generate(GenerateArgs {
            template_path: resolved,
            destination: Some(self.path.clone()),
            overwrite: self.r#override,
            init: true,
            name: Some(name.to_owned()),
            quiet: !self.verbose,
            verbose: self.verbose,
            force_git_init: true,
            lib: false,
            no_workspace: true,
            ..Default::default()
        })
        .context("can't generate project")?;

        // Create .agents/skills/cyberfabric/ directory and write SKILLS.md
        let agents_skills_dir = self.path.join(".agents").join("skills").join("cyberfabric");
        std::fs::create_dir_all(&agents_skills_dir)
            .context("failed to create .agents/skills/cyberfabric/ directory")?;
        let skills_md_path = agents_skills_dir.join("SKILL.md");
        if !skills_md_path.exists() || self.r#override {
            std::fs::write(&skills_md_path, SKILL_MD_CONTENT)
                .context("failed to write SKILL.md to .agents/skills/cyberfabric/")?;
        }

        // Dockerfile
        let docker_ignore = self.path.join(".dockerignore");
        if !docker_ignore.exists() || self.r#override {
            std::fs::write(&docker_ignore, DOCKERIGNORE_CONTENT)
                .context("failed to write .dockerignore to root directory")?;
        }
        let dockerfile_path = self.path.join("Dockerfile");
        if !dockerfile_path.exists() || self.r#override {
            std::fs::write(&dockerfile_path, DOCKERFILE_CONTENT)
                .context("failed to write Dockerfile to root directory")?;
        }

        println!("Project initialized at {}", self.path.display());
        Ok(())
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
            .unwrap_or_else(|| match self.template.as_str() {
                "default" => "Init".to_owned(),
                other => format!("Workspace/{other}"),
            });

        TemplatePath {
            git: Some(self.git.as_deref().unwrap_or(DEFAULT_GIT_URL).to_owned()),
            branch: Some(self.branch.as_deref().unwrap_or(DEFAULT_BRANCH).to_owned()),
            auto_path: Some(subfolder),
            ..TemplatePath::default()
        }
    }
}
