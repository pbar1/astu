use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::resolve::Target;

/// Factory that builds TCP connections.
#[derive(Debug, Clone, Copy)]
pub struct TransportFactory {
    connect_timeout: Duration,
}

impl TransportFactory {
    #[must_use]
    pub fn new(connect_timeout: Duration) -> Self {
        Self { connect_timeout }
    }
}

#[async_trait]
impl super::TransportFactory for TransportFactory {
    async fn setup(&self, target: &Target) -> Result<super::Transport> {
        let addr = target
            .socket_addr()
            .with_context(|| format!("unsupported target: {target}"))?;

        let tcp = timeout(self.connect_timeout, TcpStream::connect(addr))
            .await
            .context("TCP connect timed out")?
            .context("TCP connect failed")?;
        Ok(super::Transport::Tcp(tcp))
    }
}
