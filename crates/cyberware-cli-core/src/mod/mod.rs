pub mod add;

#[derive(Debug, Eq, PartialEq)]
pub struct ModArgs {
    pub command: ModCommand,
}

impl ModArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModCommand {
    Add(add::AddArgs),
}

impl ModCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Add(args) => args.run(),
        }
    }
}
