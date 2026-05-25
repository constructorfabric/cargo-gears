use crate::common::OutputFormat;

#[derive(Debug, Eq, PartialEq)]
pub struct ConfigsArgs {
    pub format: OutputFormat,
}

impl ConfigsArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        todo!(
            "list configs is blocked on the manifest-first design (Cyberware.toml); \
             requires manifest parsing to resolve app/environment links and runtime sections"
        )
    }
}
