use crate::common::OutputFormat;

#[derive(Debug, Eq, PartialEq)]
pub struct AppsParams {
    pub format: OutputFormat,
}

impl AppsParams {
    pub fn run(&self) -> anyhow::Result<()> {
        todo!(
            "list apps is blocked on the manifest-first design (Gears.toml); \
             requires manifest parsing to enumerate apps, environments, and build outputs"
        )
    }
}
