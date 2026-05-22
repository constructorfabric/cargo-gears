use clap::Parser;
use cyberware_cli::Cli;

// cargo invokes this binary as `cargo-cyberfabric cyberfabric <args>`
// so the parser below is defined with that in mind
#[derive(Parser)]
#[clap(bin_name = "cargo")]
enum Opt {
    Cyberfabric(Cli),
}

fn main() -> anyhow::Result<()> {
    let Opt::Cyberfabric(cargo) = Opt::parse();
    cargo.run()
}
