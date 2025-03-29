use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;

mod opaque;
mod tcp;
mod tcp_reuse;

pub use opaque::OpaqueTransportFactory;
pub use tcp::TcpTransportFactory;
pub use tcp_reuse::TcpReuseTransportFactory;

/// Bytestream transports that will be used by clients to connect to targets.
pub enum Transport {
    /// No stream, will be handled by the client.
    Opaque,
    /// Async TCP stream.
    Tcp(tokio::net::TcpStream),
}

/// Factory for creating transports.
#[async_trait]
pub trait TransportFactory {
    /// Connect to a target and return the transport stream.
    async fn connect(&self, target: &Target) -> Result<Transport>;
}
