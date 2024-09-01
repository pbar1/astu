use clap::Args;

/// Run commands on targets
#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Target query
    query: String,
}

#[async_trait::async_trait]
impl super::Run for ExecArgs {
    async fn run(&self) -> anyhow::Result<()> {
        println!("{}", self.query);
        Ok(())
    }
}
