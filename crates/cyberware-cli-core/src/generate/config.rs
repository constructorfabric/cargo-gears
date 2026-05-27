use anyhow::{Context, bail};
use std::fs;
use std::path::PathBuf;

/// Built-in config template names
const BUILTIN_DEV: &str = "dev";
const BUILTIN_PROD: &str = "prod";
const BUILTIN_DB: &str = "db";

const DEV_TEMPLATE: &str = r"server:
  home_dir: .cyberfabric

logging:
  default:
    console_level: info
    file: logs/cyberfabric.log
    file_level: debug
    max_age_days: 7
    max_backups: 3
    max_size_mb: 100

modules: {}
";

const PROD_TEMPLATE: &str = r"server:
  home_dir: .cyberfabric

logging:
  default:
    console_level: warn
    file: logs/cyberfabric.log
    file_level: info
    max_age_days: 30
    max_backups: 10
    max_size_mb: 500

opentelemetry:
  resource:
    service_name: cyberfabric
  tracing:
    enabled: true
    sampler:
      parent_based_ratio:
        ratio: 0.1
  metrics:
    enabled: true

modules: {}
";

const DB_TEMPLATE: &str = r"server:
  home_dir: .cyberfabric

logging:
  default:
    console_level: info
    file: logs/cyberfabric.log
    file_level: debug
    max_age_days: 7
    max_backups: 3
    max_size_mb: 100

database:
  servers:
    main:
      engine: postgres
      host: localhost
      port: 5432
      user: postgres
      password: ${DB_PASSWORD}
      dbname: app

modules: {}
";

#[derive(Debug, Eq, PartialEq)]
pub struct GenerateConfigArgs {
    /// Config template to use: dev, prod, or db.
    pub template: String,
    /// Application name for the output filename.
    pub app: Option<String>,
    /// Environment name for the output filename.
    pub env: Option<String>,
    /// Custom output filename (overrides app/env naming).
    pub name: Option<String>,
    /// Workspace root path.
    pub path: PathBuf,
}

impl GenerateConfigArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let content = self.resolve_template_content()?;
        let output_path = self.resolve_output_path();

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("can't create directory {}", parent.display()))?;
        }

        if output_path.exists() {
            bail!(
                "config file already exists at {}. Remove it first or use a different name.",
                output_path.display()
            );
        }

        fs::write(&output_path, content)
            .with_context(|| format!("can't write config file {}", output_path.display()))?;

        println!("Config created at {}", output_path.display());
        Ok(())
    }

    fn resolve_template_content(&self) -> anyhow::Result<String> {
        match self.template.as_str() {
            BUILTIN_DEV => Ok(DEV_TEMPLATE.to_owned()),
            BUILTIN_PROD => Ok(PROD_TEMPLATE.to_owned()),
            BUILTIN_DB => Ok(DB_TEMPLATE.to_owned()),
            custom => {
                bail!(
                    "unknown config template '{custom}'. Available built-in templates: dev, prod, db"
                );
            }
        }
    }

    fn resolve_output_path(&self) -> PathBuf {
        let filename = self.resolve_filename();
        self.path.join("config").join(filename)
    }

    fn resolve_filename(&self) -> String {
        if let Some(name) = &self.name {
            return ensure_yml_extension(name);
        }

        match (&self.app, &self.env) {
            (Some(app), Some(env)) => format!("{app}-{env}.yml"),
            (Some(app), None) => format!("{app}.yml"),
            (None, Some(env)) => format!("{env}.yml"),
            (None, None) => format!("{}.yml", self.template),
        }
    }
}

fn ensure_yml_extension(name: &str) -> String {
    let path = std::path::Path::new(name);
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
    {
        name.to_owned()
    } else if let Some(file_name) = path.file_name() {
        format!("{}.yml", file_name.display())
    } else {
        name.split('.').next().unwrap_or(name).to_owned() + ".yml"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_defaults_to_template_name() {
        let args = GenerateConfigArgs {
            template: "dev".to_owned(),
            app: None,
            env: None,
            name: None,
            path: PathBuf::from("."),
        };
        assert_eq!(args.resolve_filename(), "dev.yml");
    }

    #[test]
    fn filename_uses_app_and_env() {
        let args = GenerateConfigArgs {
            template: "dev".to_owned(),
            app: Some("myapp".to_owned()),
            env: Some("staging".to_owned()),
            name: None,
            path: PathBuf::from("."),
        };
        assert_eq!(args.resolve_filename(), "myapp-staging.yml");
    }

    #[test]
    fn filename_uses_custom_name() {
        let args = GenerateConfigArgs {
            template: "dev".to_owned(),
            app: Some("ignored".to_owned()),
            env: Some("ignored".to_owned()),
            name: Some("custom".to_owned()),
            path: PathBuf::from("."),
        };
        assert_eq!(args.resolve_filename(), "custom.yml");
    }

    #[test]
    fn filename_preserves_yml_extension() {
        let args = GenerateConfigArgs {
            template: "dev".to_owned(),
            app: None,
            env: None,
            name: Some("already.yml".to_owned()),
            path: PathBuf::from("."),
        };
        assert_eq!(args.resolve_filename(), "already.yml");
    }

    #[test]
    fn dev_template_is_valid_yaml() {
        let _: crate::app_config::AppConfig =
            serde_saphyr::from_str(DEV_TEMPLATE).expect("dev template should be valid YAML");
    }

    #[test]
    fn prod_template_is_valid_yaml() {
        let _: crate::app_config::AppConfig =
            serde_saphyr::from_str(PROD_TEMPLATE).expect("prod template should be valid YAML");
    }

    #[test]
    fn db_template_is_valid_yaml() {
        let _: crate::app_config::AppConfig =
            serde_saphyr::from_str(DB_TEMPLATE).expect("db template should be valid YAML");
    }
}
