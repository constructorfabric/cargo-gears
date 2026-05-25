use clap::Parser;
use cyberware_cli::Cli;
use cyberware_cli_core::CyberfabricCommand;

#[allow(clippy::expect_used)]
pub fn parse_command(args: &[&str]) -> CyberfabricCommand {
    Cli::try_parse_from(args).expect("argv should parse").into()
}
