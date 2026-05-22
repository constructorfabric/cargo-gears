use super::{load_config, resolve_modules_context, save_config, validate_module_name};
use crate::common::PathConfigArgs;
use anyhow::bail;

pub struct RemoveArgs {
    pub path_config: PathConfigArgs,
    /// Module name
    pub module: String,
}

impl RemoveArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        validate_module_name(&self.module)?;
        let context = resolve_modules_context(&self.path_config)?;

        let mut config = load_config(&context.config_path)?;
        if config.modules.remove(&self.module).is_none() {
            let module = &self.module;
            bail!("module '{module}' not found in modules section");
        }

        save_config(&context.config_path, &config)
    }
}
