use super::{load_config, save_config, validate_module_name};
use crate::common::PathConfigArgs;
use anyhow::bail;

#[derive(Debug, Eq, PartialEq)]
pub struct RemoveArgs {
    pub path_config: PathConfigArgs,
    /// Module name
    pub module: String,
}

impl RemoveArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.path_config.with_workspace_dir(|config_path| {
            validate_module_name(&self.module)?;

            let mut config = load_config(config_path)?;
            if config.modules.remove(&self.module).is_none() {
                let module = &self.module;
                bail!("module '{module}' not found in modules section");
            }

            save_config(config_path, &config)
        })
    }
}
