mod default;
mod reuseport;

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::net::TcpStream;

pub use crate::tcp_stream::default::DefaultTcpStreamFactory;
pub use crate::tcp_stream::reuseport::ReuseportTcpStreamFactory;

/// Factory for creating Tokio TCP streams.
#[async_trait]
pub trait TcpStreamFactory {
    async fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> Result<TcpStream>;
}
