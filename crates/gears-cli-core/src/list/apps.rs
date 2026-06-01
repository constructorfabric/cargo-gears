use crate::common::OutputFormat;
use crate::manifest::{self, Manifest};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct AppsParams {
    pub path: Option<PathBuf>,
    pub manifest: PathBuf,
    pub format: OutputFormat,
}

#[derive(Debug, Serialize)]
struct AppEntry {
    app: String,
    env: String,
    generated_name: String,
}

impl AppsParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_root = crate::common::resolve_workspace_path(self.path.as_deref())?;
        let manifest_path = manifest::resolve_manifest_path(&workspace_root, &self.manifest)?;
        let manifest = Manifest::load(&manifest_path)?;

        let mut entries: Vec<AppEntry> = Vec::new();
        for (app, envs) in &manifest.apps {
            for env in envs.keys() {
                match manifest.resolve(&workspace_root, &manifest_path, app, env, None) {
                    Ok(resolved) => entries.push(AppEntry {
                        app: app.clone(),
                        env: env.clone(),
                        generated_name: resolved.generated_name,
                    }),
                    Err(err) => {
                        eprintln!("warning: {app}/{env}: {err}");
                    }
                }
            }
        }

        match self.format {
            OutputFormat::Table => {
                println!("Apps:");
                if entries.is_empty() {
                    println!("  (none)");
                } else {
                    for entry in &entries {
                        println!(
                            "  - {}/{} (build: {})",
                            entry.app, entry.env, entry.generated_name
                        );
                    }
                }
                Ok(())
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&entries)?);
                Ok(())
            }
        }
    }
}
