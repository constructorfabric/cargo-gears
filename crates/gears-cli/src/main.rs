use clap::Parser;
use gears_cli::Cli;

// cargo invokes this binary as `cargo-gears gears <args>`
// so the parser below is defined with that in mind
#[derive(Parser)]
#[clap(bin_name = "cargo")]
enum Opt {
    Gears(Cli),
}

fn main() -> anyhow::Result<()> {
    let Opt::Gears(cargo) = Opt::parse();
    cargo.run()
}
