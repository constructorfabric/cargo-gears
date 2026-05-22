use clap::{Args, ValueEnum};

#[derive(Args)]
pub struct ToolsArgs {
    /// Install all tools
    #[arg(short = 'a', long, conflicts_with = "install")]
    all: bool,
    /// Upgrade tools to the recommended version
    #[arg(short = 'u', long)]
    upgrade: bool,
    /// Install specific tools
    #[arg(long, value_delimiter = ',', value_enum, conflicts_with = "all")]
    install: Option<Vec<ToolName>>,
    /// Do not ask for confirmation
    #[arg(short = 'y', long)]
    yolo: bool,
    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

impl ToolsArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::tools::ToolsArgs {
            all: self.all,
            upgrade: self.upgrade,
            install: self
                .install
                .map(|tools| tools.into_iter().map(Into::into).collect()),
            yolo: self.yolo,
            verbose: self.verbose,
        }
        .run()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum ToolName {
    Rustup,
    Rustfmt,
    Clippy,
}

impl From<ToolName> for cyberware_cli_core::tools::ToolName {
    fn from(tool: ToolName) -> Self {
        match tool {
            ToolName::Rustup => Self::Rustup,
            ToolName::Rustfmt => Self::Rustfmt,
            ToolName::Clippy => Self::Clippy,
        }
    }
}
