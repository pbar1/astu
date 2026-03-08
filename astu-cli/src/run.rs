pub trait Run {
    async fn run(&self) -> anyhow::Result<()>;
}
