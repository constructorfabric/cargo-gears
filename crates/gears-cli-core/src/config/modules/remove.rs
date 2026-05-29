use super::{load_config, save_config, validate_module_name};
use crate::common::PathConfigParams;
use anyhow::bail;

#[derive(Debug, Eq, PartialEq)]
pub struct RemoveParams {
    pub path_config: PathConfigParams,
    /// Module name
    pub module: String,
}

impl RemoveParams {
    pub fn run(&self) -> anyhow::Result<()> {
        self.path_config
            .with_workspace_dir(|_workspace_path, config_path| {
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
