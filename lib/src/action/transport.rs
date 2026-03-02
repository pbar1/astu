//! Underlying transport used by clients.

use std::net::SocketAddr;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

use crate::resolve::Target;
use crate::resolve::TargetKind;

pub mod opaque;
pub mod tcp;
pub mod tcp_reuse;

/// Bytestream transports that will be used by clients to connect to targets.
#[non_exhaustive]
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

pub(crate) async fn socket_addr_for_target(target: &Target) -> Result<SocketAddr> {
    if let Some(addr) = target.socket_addr() {
        return Ok(addr);
    }

    if target.kind() != TargetKind::Ssh {
        bail!("unsupported target: {target}");
    }

    let port = target.port().unwrap_or(22);
    if let Some(ip) = target.ip() {
        return Ok(SocketAddr::new(ip, port));
    }

    if let Some(domain) = target.domain() {
        let mut addrs = tokio::net::lookup_host((domain, port))
            .await
            .with_context(|| format!("failed resolving ssh target: {target}"))?;
        if let Some(addr) = addrs.next() {
            return Ok(addr);
        }
    }

    bail!("unsupported target: {target}")
}
