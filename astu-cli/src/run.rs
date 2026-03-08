pub trait Run {
    async fn run(&self) -> eyre::Result<()>;
}
