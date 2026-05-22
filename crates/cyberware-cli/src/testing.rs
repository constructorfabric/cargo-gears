use clap::Args;

#[derive(Args)]
pub struct TestArgs {
    #[arg(long)]
    e2e: bool,
    #[arg(long)]
    module: Option<String>,
    #[arg(long)]
    coverage: bool,
}

impl TestArgs {
    pub fn run(self) -> anyhow::Result<()> {
        cyberware_cli_core::test::TestArgs {
            e2e: self.e2e,
            module: self.module,
            coverage: self.coverage,
        }
        .run()
    }
}
