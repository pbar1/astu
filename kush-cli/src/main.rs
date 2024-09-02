mod cli;
mod mapper;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
