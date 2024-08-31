use std::net::SocketAddr;
use std::time::Duration;

/// Simply calls `connect` without doing anything special
pub struct DefaultTcpFactory;

impl super::TcpFactory for DefaultTcpFactory {
    fn connect_timeout(
        &self,
        addr: &SocketAddr,
        timeout: Duration,
    ) -> anyhow::Result<std::net::TcpStream> {
        let stream = std::net::TcpStream::connect_timeout(addr.into(), timeout)?;
        Ok(stream)
    }
}

#[async_trait::async_trait]
impl super::TcpFactoryAsync for DefaultTcpFactory {
    async fn connect_timeout_async(
        &self,
        addr: &SocketAddr,
        timeout: Duration,
    ) -> anyhow::Result<tokio::net::TcpStream> {
        let connect = tokio::net::TcpStream::connect(addr);
        let stream = tokio::time::timeout(timeout, connect).await??;
        Ok(stream)
    }
}
