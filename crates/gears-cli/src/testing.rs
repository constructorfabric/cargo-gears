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
        gears_cli_core::test::TestArgs::from(self).run()
    }
}

impl From<TestArgs> for gears_cli_core::test::TestArgs {
    fn from(args: TestArgs) -> Self {
        Self {
            e2e: args.e2e,
            module: args.module,
            coverage: args.coverage,
        }
    }
}
