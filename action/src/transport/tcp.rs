use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::Transport;
use super::TransportFactory;

/// Factory that builds TCP connections.
#[derive(Debug, Clone, Copy)]
pub struct TcpTransportFactory {
    connect_timeout: Duration,
}

impl TcpTransportFactory {
    pub fn new(connect_timeout: Duration) -> Self {
        Self { connect_timeout }
    }
}

#[async_trait]
impl TransportFactory for TcpTransportFactory {
    async fn connect(&self, target: &Target) -> Result<Transport> {
        let addr = match target {
            Target::SocketAddr(addr) => *addr,
            unsupported => bail!("TcpTransportFactory: unsupported target: {unsupported}"),
        };
        let tcp = timeout(self.connect_timeout, TcpStream::connect(addr))
            .await
            .context("TCP connect timed out")?
            .context("TCP connect failed")?;
        Ok(Transport::Tcp(tcp))
    }
}
