mod argetype;
mod cmd;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cmd::run().await
}
