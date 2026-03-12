//! Underlying transport used by clients.

use astu_types::Target;
use eyre::Result;

pub mod tcp;
pub mod tcp_reuse;

/// Bytestream transports that will be used by clients to connect to targets.
#[non_exhaustive]
#[derive(Debug)]
pub enum Transport {
    /// Async TCP stream.
    Tcp(tokio::net::TcpStream),
}

/// Factory for creating transports.
pub trait TransportFactory {
    /// Sets up a transport to the target.
    async fn setup(&self, target: &Target) -> Result<Transport>;
}

/// All transport factory implementations.
#[derive(Debug, Clone)]
pub enum TransportFactoryImpl {
    Tcp(tcp::TransportFactory),
    TcpReuse(tcp_reuse::TransportFactory),
}

impl TransportFactory for TransportFactoryImpl {
    async fn setup(&self, target: &Target) -> Result<Transport> {
        match self {
            Self::Tcp(factory) => factory.setup(target).await,
            Self::TcpReuse(factory) => factory.setup(target).await,
        }
    }
}
