use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use astu_util::tcp::DefaultTcpFactory;
use astu_util::tcp::TcpFactoryAsync;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = SocketAddr::from_str("10.0.0.54:22")?;

    let tcp = DefaultTcpFactory
        .connect_timeout_async(&addr, Duration::from_secs(2))
        .await?;

    let mut reader = BufReader::new(tcp);

    let mut server_id = String::new();
    reader.read_line(&mut server_id).await?;
    let server_id = server_id.trim();

    println!("{server_id}");

    Ok(())
}
