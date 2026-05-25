use crate::common::OutputFormat;

#[derive(Debug, Eq, PartialEq)]
pub struct AppsArgs {
    pub format: OutputFormat,
}

impl AppsArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        todo!(
            "list apps is blocked on the manifest-first design (Cyberware.toml); \
             requires manifest parsing to enumerate apps, environments, and build outputs"
        )
    }
}
