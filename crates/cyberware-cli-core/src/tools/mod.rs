use anyhow::{Context, bail};
use std::io::{self, Write};
use std::process::Command;
use std::{fmt, slice};

#[derive(Debug, Eq, PartialEq)]
pub struct ToolsArgs {
    /// Install all tools
    pub all: bool,
    /// Upgrade tools to the recommended version
    pub upgrade: bool,
    /// Install specific tools
    pub install: Option<Vec<ToolName>>,
    /// Do not ask for confirmation
    pub yolo: bool,
    /// Verbose output
    pub verbose: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolName {
    Rustup,
    Rustfmt,
    Clippy,
}

impl ToolName {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rustup => "rustup",
            Self::Rustfmt => "rustfmt",
            Self::Clippy => "clippy",
        }
    }

    const fn check_binary(self) -> &'static str {
        match self {
            Self::Rustup => "rustup",
            Self::Rustfmt => "rustfmt",
            Self::Clippy => "cargo-clippy",
        }
    }

    const fn install_method(self) -> InstallMethod {
        match self {
            Self::Rustup => InstallMethod::Prerequisite,
            Self::Rustfmt => InstallMethod::RustupComponent("rustfmt"),
            Self::Clippy => InstallMethod::RustupComponent("clippy"),
        }
    }

    fn all() -> slice::Iter<'static, Self> {
        ALL_TOOLS.iter()
    }
}

impl fmt::Display for ToolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy)]
enum InstallMethod {
    RustupComponent(&'static str),
    Prerequisite,
}

const ALL_TOOLS: &[ToolName] = &[ToolName::Rustup, ToolName::Rustfmt, ToolName::Clippy];

impl ToolsArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let tools = self.resolve_tools()?;

        if self.upgrade {
            return self.upgrade_tools(&tools);
        }

        self.install_tools(&tools)
    }

    fn resolve_tools(&self) -> anyhow::Result<Vec<ToolName>> {
        if let Some(tools) = &self.install {
            return Ok(tools.clone());
        }

        if self.all {
            return Ok(ToolName::all().copied().collect());
        }

        bail!(
            "no tools specified. Use --all to install all tools, or --install <tool,...> to install specific tools. \
             Known tools: {}",
            ToolName::all()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn install_tools(&self, tools: &[ToolName]) -> anyhow::Result<()> {
        ensure_rustup(self.yolo)?;

        for tool in tools {
            let installed = is_installed(tool.check_binary());
            if installed {
                println!("✓ {tool} is already installed");
                continue;
            }

            match tool.install_method() {
                InstallMethod::Prerequisite => {
                    bail!(
                        "'{tool}' is required but not found. Please install it manually: https://rustup.rs"
                    );
                }
                InstallMethod::RustupComponent(component) => {
                    if !self.yolo && !confirm(&format!("Install {tool} via rustup?"))? {
                        println!("Skipping {tool}");
                        continue;
                    }
                    rustup_component_add(component, self.verbose)?;
                    println!("✓ {tool} installed");
                }
            }
        }

        Ok(())
    }

    fn upgrade_tools(&self, tools: &[ToolName]) -> anyhow::Result<()> {
        ensure_rustup(self.yolo)?;

        let has_rustup = tools.contains(&ToolName::Rustup);
        if has_rustup {
            if !self.yolo && !confirm("Upgrade rustup via 'rustup self update'?")? {
                println!("Skipping rustup upgrade");
            } else {
                run_verbose(
                    Command::new("rustup").arg("self").arg("update"),
                    self.verbose,
                )
                .context("failed to upgrade rustup")?;
                println!("✓ rustup upgraded");
            }
        }

        let components: Vec<_> = tools
            .iter()
            .copied()
            .filter(|tool| matches!(tool.install_method(), InstallMethod::RustupComponent(_)))
            .collect();

        if !components.is_empty() {
            if !self.yolo && !confirm("Upgrade rustup components via 'rustup update'?")? {
                println!("Skipping component upgrades");
                return Ok(());
            }
            run_verbose(Command::new("rustup").arg("update"), self.verbose)
                .context("failed to run rustup update")?;
            for tool in components {
                println!("✓ {tool} upgraded");
            }
        }

        Ok(())
    }
}

fn ensure_rustup(yolo: bool) -> anyhow::Result<()> {
    if is_installed("rustup") {
        return Ok(());
    }

    if !yolo && !confirm("rustup is not installed. Install it now?")? {
        bail!(
            "rustup is required but not installed. \
             Please install it manually (https://rustup.rs) or re-run with --yolo to auto-install."
        );
    }

    println!("Installing rustup...");
    install_rustup().context("failed to install rustup")?;
    println!("✓ rustup installed");
    Ok(())
}

fn install_rustup() -> anyhow::Result<()> {
    if cfg!(target_family = "unix") {
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y")
            .status()
            .context("failed to run rustup installer (is curl installed?)")?;
        if !status.success() {
            bail!("rustup installer exited with {status}");
        }
    } else if cfg!(target_family = "windows") {
        let tmp = std::env::temp_dir().join("rustup-init.exe");
        let status = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile '{}'",
                    tmp.display()
                ),
            ])
            .status()
            .context("failed to download rustup-init.exe")?;
        if !status.success() {
            bail!("failed to download rustup-init.exe (exit {status})");
        }
        let status = Command::new(&tmp)
            .arg("-y")
            .status()
            .context("failed to run rustup-init.exe")?;
        if !status.success() {
            bail!("rustup-init.exe exited with {status}");
        }
    } else {
        bail!("unsupported platform. Please install rustup manually: https://rustup.rs");
    }

    Ok(())
}

fn is_installed(binary: &str) -> bool {
    Command::new(binary)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn rustup_component_add(component: &str, verbose: bool) -> anyhow::Result<()> {
    run_verbose(
        Command::new("rustup")
            .arg("component")
            .arg("add")
            .arg(component),
        verbose,
    )
    .with_context(|| format!("failed to install rustup component '{component}'"))
}

fn run_verbose(cmd: &mut Command, verbose: bool) -> anyhow::Result<()> {
    if !verbose {
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
    }
    let status = cmd.status().context("failed to execute command")?;
    if !status.success() {
        bail!("command exited with {status}");
    }
    Ok(())
}

fn confirm(prompt: &str) -> anyhow::Result<bool> {
    print!("{prompt} [Y/n] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y" || trimmed == "yes")
}
