use crate::common::OutputFormat;
use crate::manifest::{self, Manifest};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct ConfigsParams {
    pub path: Option<PathBuf>,
    pub manifest: PathBuf,
    pub format: OutputFormat,
}

#[derive(Debug, Serialize)]
struct ConfigEntry {
    app: String,
    env: String,
    config_path: PathBuf,
}

impl ConfigsParams {
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_root = crate::common::resolve_workspace_path(self.path.as_deref())?;
        let manifest_path = manifest::resolve_manifest_path(&workspace_root, &self.manifest)?;
        let manifest = Manifest::load(&manifest_path)?;

        let mut entries: Vec<ConfigEntry> = Vec::new();
        for (app, envs) in &manifest.apps {
            for env in envs.keys() {
                match manifest.resolve(&workspace_root, &manifest_path, app, env, None) {
                    Ok(resolved) => entries.push(ConfigEntry {
                        app: app.clone(),
                        env: env.clone(),
                        config_path: resolved.config_path,
                    }),
                    Err(err) => {
                        eprintln!("warning: {app}/{env}: {err}");
                    }
                }
            }
        }

        match self.format {
            OutputFormat::Table => {
                println!("Config files:");
                if entries.is_empty() {
                    println!("  (none)");
                } else {
                    for entry in &entries {
                        println!(
                            "  - {}/{}: {}",
                            entry.app,
                            entry.env,
                            entry.config_path.display()
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
