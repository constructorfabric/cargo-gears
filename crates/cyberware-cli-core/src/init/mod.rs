use anyhow::{Context, bail};
use cargo_generate::{GenerateArgs, TemplatePath, generate};
use std::path::PathBuf;

/// Content of SKILL.md embedded at compile time
const SKILL_MD_CONTENT: &str = include_str!("../../../../SKILL.md");

/// Content of Dockerfile embedded at compile time
const DOCKERFILE_CONTENT: &str = include_str!("../../shared/Dockerfile");

/// Content of .dockerignore embedded at compile time
const DOCKERIGNORE_CONTENT: &str = include_str!("../../shared/.dockerignore");

#[derive(Debug, Eq, PartialEq)]
pub struct InitArgs {
    /// Path to initialize the project
    pub path: PathBuf,
    /// Name of the project, it's inferred from the path name if not specified
    pub name: Option<String>,
    /// Verbose output
    pub verbose: bool,
    /// Path to a local template (instead of git)
    pub local_path: Option<String>,
    /// url to the git repo
    pub git: Option<String>,
    /// Subfolder relative to the git repo
    pub subfolder: Option<String>,
    /// Branch of the git repo
    pub branch: Option<String>,
    pub r#override: bool,
}

impl InitArgs {
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
        let (git, branch) = if self.local_path.is_some() {
            (None, None)
        } else {
            (self.git.clone(), self.branch.clone())
        };
        generate(GenerateArgs {
            template_path: TemplatePath {
                auto_path: self.subfolder.clone(),
                git,
                path: self.local_path.clone(),
                subfolder: None, // This is only used when git, path and favorite are not specified
                branch,
                tag: None,
                test: false,
                revision: None,
                favorite: None,
            },
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
}
