pub struct TestArgs {
    pub e2e: bool,
    pub module: Option<String>,
    pub coverage: bool,
}

impl TestArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
