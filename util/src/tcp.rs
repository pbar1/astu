mod default;
mod reuseport;

pub use crate::tcp::default::DefaultTcpFactory;
pub use crate::tcp::reuseport::ReuseportTcpFactory;

/// Factory for creating standard library TCP streams.
pub trait TcpFactory {
    fn connect_timeout(
        &self,
        addr: &std::net::SocketAddr,
        timeout: std::time::Duration,
    ) -> anyhow::Result<std::net::TcpStream>;
}

/// Factory for creating Tokio TCP streams.
#[async_trait::async_trait]
pub trait TcpFactoryAsync {
    async fn connect_timeout_async(
        &self,
        addr: &std::net::SocketAddr,
        timeout: std::time::Duration,
    ) -> anyhow::Result<tokio::net::TcpStream>;
}
