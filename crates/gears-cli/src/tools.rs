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
        gears_cli_core::tools::ToolsArgs::from(self).run()
    }
}

impl From<ToolsArgs> for gears_cli_core::tools::ToolsArgs {
    fn from(args: ToolsArgs) -> Self {
        Self {
            all: args.all,
            upgrade: args.upgrade,
            install: args
                .install
                .map(|tools| tools.into_iter().map(Into::into).collect()),
            yolo: args.yolo,
            verbose: args.verbose,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum ToolName {
    Rustup,
    Rustfmt,
    Clippy,
}

impl From<ToolName> for gears_cli_core::tools::ToolName {
    fn from(tool: ToolName) -> Self {
        match tool {
            ToolName::Rustup => Self::Rustup,
            ToolName::Rustfmt => Self::Rustfmt,
            ToolName::Clippy => Self::Clippy,
        }
    }
}
