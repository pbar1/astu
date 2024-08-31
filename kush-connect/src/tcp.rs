mod default;
mod reuseport;

use std::net::SocketAddr;
use std::time::Duration;

/// Factory for creating standard library TCP streams.
pub trait TcpFactory {
    fn connect_timeout(
        &self,
        addr: &SocketAddr,
        timeout: Duration,
    ) -> anyhow::Result<std::net::TcpStream>;
}

/// Factory for creating Tokio TCP streams.
#[async_trait::async_trait]
pub trait TcpFactoryAsync {
    async fn connect_timeout_async(
        &self,
        addr: &SocketAddr,
        timeout: Duration,
    ) -> anyhow::Result<tokio::net::TcpStream>;
}
