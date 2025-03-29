use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Factory that builds TCP connections.
#[derive(Debug, Clone, Copy)]
pub struct TransportFactory {
    connect_timeout: Duration,
}

impl TransportFactory {
    pub fn new(connect_timeout: Duration) -> Self {
        Self { connect_timeout }
    }
}

#[async_trait]
impl super::TransportFactory for TransportFactory {
    async fn setup(&self, target: &Target) -> Result<super::Transport> {
        let addr = match target {
            Target::SocketAddr(addr) => *addr,
            unsupported => bail!("TcpTransportFactory: unsupported target: {unsupported}"),
        };
        let tcp = timeout(self.connect_timeout, TcpStream::connect(addr))
            .await
            .context("TCP connect timed out")?
            .context("TCP connect failed")?;
        Ok(super::Transport::Tcp(tcp))
    }
}
