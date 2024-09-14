use std::net::SocketAddr;
use std::time::Duration;

use crate::tcp_stream::TcpStreamFactory;

/// Simply calls `connect` without doing anything special
pub struct DefaultTcpStreamFactory;

#[async_trait::async_trait]
impl TcpStreamFactory for DefaultTcpStreamFactory {
    async fn connect_timeout(
        &self,
        addr: &SocketAddr,
        timeout: Duration,
    ) -> anyhow::Result<tokio::net::TcpStream> {
        let connect = tokio::net::TcpStream::connect(addr);
        let stream = tokio::time::timeout(timeout, connect).await??;
        Ok(stream)
    }
}
