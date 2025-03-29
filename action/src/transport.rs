//! Underlying transport used by clients.

use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

pub mod opaque;
pub mod tcp;
pub mod tcp_reuse;

/// Bytestream transports that will be used by clients to connect to targets.
#[derive(Debug)]
pub enum Transport {
    /// No stream, will be handled by the client.
    Opaque,
    /// Async TCP stream.
    Tcp(tokio::net::TcpStream),
}

/// Factory for creating transports.
#[async_trait]
#[enum_dispatch]
pub trait TransportFactory {
    /// Sets up a transport to the target.
    async fn setup(&self, target: &Target) -> Result<Transport>;
}

/// All transport factory implementations.
#[enum_dispatch(TransportFactory)]
#[derive(Debug, Clone)]
pub enum TransportFactoryImpl {
    Opaque(opaque::TransportFactory),
    Tcp(tcp::TransportFactory),
    TcpReuse(tcp_reuse::TransportFactory),
}
