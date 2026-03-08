use std::time::Duration;

use astu_types::Target;
use async_trait::async_trait;
use eyre::Result;
use eyre::WrapErr;
use eyre::eyre;
use tokio::net::TcpStream;
use tokio::time::timeout;

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
            .ok_or_else(|| eyre!("unsupported target: {target}"))?;

        let tcp = timeout(self.connect_timeout, TcpStream::connect(addr))
            .await
            .wrap_err("TCP connect timed out")?
            .wrap_err("TCP connect failed")?;
        Ok(super::Transport::Tcp(tcp))
    }
}
