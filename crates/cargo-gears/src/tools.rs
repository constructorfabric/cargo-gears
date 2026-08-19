use clap::{Args, Subcommand, ValueEnum};

#[derive(Args)]
pub struct ToolsArgs {
    #[command(subcommand)]
    command: Option<ToolsCommand>,

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

#[derive(Subcommand)]
enum ToolsCommand {
    /// Check that a tool satisfies a version requirement.
    /// Runs `<tool> --version`, parses the version, and checks against <requirement>.
    /// Exits 0 if satisfied, 1 if not.
    CheckVersion(CheckVersionArgs),
}

#[derive(Args)]
struct CheckVersionArgs {
    /// Tool binary name (e.g. cargo-deny, cargo-nextest)
    tool: String,
    /// Semver requirement (e.g. '>=0.20.0', '^0.9.130')
    requirement: String,
}

impl ToolsArgs {
    pub fn run(self) -> anyhow::Result<()> {
        if let Some(ToolsCommand::CheckVersion(args)) = self.command {
            return run_check_version(&args.tool, &args.requirement);
        }
        cargo_gears_core::tools::ToolsParams::from(self).run()
    }
}

fn run_check_version(tool: &str, req_str: &str) -> anyhow::Result<()> {
    let requirement = semver::VersionReq::parse(req_str)
        .map_err(|e| anyhow::anyhow!("invalid version requirement '{req_str}': {e}"))?;

    let output = std::process::Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|_| anyhow::anyhow!("{tool} is not installed"))?;

    if !output.status.success() {
        anyhow::bail!("{tool} --version exited with {}", output.status);
    }

    // Combine stdout and stderr: some tools write version info to stderr.
    let version_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    // Parse version from output: try each whitespace-separated token
    let version = version_output
        .split_whitespace()
        .find_map(|token| semver::Version::parse(token.trim_end_matches(',')).ok());

    let Some(version) = version else {
        anyhow::bail!(
            "cannot parse version from `{tool} --version` output: {}",
            version_output.trim()
        );
    };

    if requirement.matches(&version) {
        println!("{version}");
        Ok(())
    } else {
        anyhow::bail!("{tool} {version} does not satisfy {req_str}");
    }
}

impl From<ToolsArgs> for cargo_gears_core::tools::ToolsParams {
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

impl From<ToolName> for cargo_gears_core::tools::ToolName {
    fn from(tool: ToolName) -> Self {
        match tool {
            ToolName::Rustup => Self::Rustup,
            ToolName::Rustfmt => Self::Rustfmt,
            ToolName::Clippy => Self::Clippy,
        }
    }
}
