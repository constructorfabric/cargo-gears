#[derive(Debug, Eq, PartialEq)]
pub struct TestParams {
    pub e2e: bool,
    pub module: Option<String>,
    pub coverage: bool,
}

impl TestParams {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
