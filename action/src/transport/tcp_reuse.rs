use std::net::SocketAddr;
use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;
use tokio::net::TcpSocket;
use tokio::time::timeout;

use super::Transport;
use super::TransportFactory;

/// Factory that builds TCP connections all sharing a local address.
///
/// This gets around the default behavior of allocating a new port for each
/// outgoing connection, at the expense of each connection being made unique
/// only by the remote address. In other words, each remote target can only be
/// conncted to by at most one transport created by each instance of this
/// factory.
#[derive(Debug)]
pub struct TcpReuseTransportFactory {
    connect_timeout: Duration,
    reserved_v4: TcpSocket,
    reserved_v6: TcpSocket,
}

impl TcpReuseTransportFactory {
    pub fn try_new(connect_timeout: Duration) -> Result<Self> {
        let reserved_v4 =
            reserve_socket_v4().context("failed reserving local v4 socket address")?;
        let reserved_v6 =
            reserve_socket_v6().context("failed reserving local v6 socket address")?;
        Ok(Self {
            connect_timeout,
            reserved_v4,
            reserved_v6,
        })
    }
}

#[async_trait]
impl TransportFactory for TcpReuseTransportFactory {
    async fn connect(&self, target: &Target) -> Result<Transport> {
        let addr = match target {
            Target::SocketAddr(addr) => *addr,
            unsupported => bail!("TcpTransportFactory: unsupported target: {unsupported}"),
        };

        let local_addr = match addr {
            SocketAddr::V4(_) => self
                .reserved_v4
                .local_addr()
                .context("unable to get local v4 socket addr")?,
            SocketAddr::V6(_) => self
                .reserved_v6
                .local_addr()
                .context("unable to get local v6 socket addr")?,
        };

        let socket =
            new_reuseport_socket(local_addr).context("unable to build local reusable socket")?;

        let tcp = timeout(self.connect_timeout, socket.connect(addr))
            .await
            .context("TCP connect timed out")?
            .context("TCP connect failed")?;
        Ok(Transport::Tcp(tcp))
    }
}

fn reserve_socket_v4() -> Result<TcpSocket> {
    let ip = std::net::Ipv4Addr::UNSPECIFIED;
    let unspec = std::net::SocketAddrV4::new(ip, 0);
    new_reuseport_socket(SocketAddr::from(unspec))
}

fn reserve_socket_v6() -> Result<TcpSocket> {
    let ip = std::net::Ipv6Addr::UNSPECIFIED;
    let unspec = std::net::SocketAddrV6::new(ip, 0, 0, 0);
    new_reuseport_socket(SocketAddr::from(unspec))
}

fn new_reuseport_socket(local_addr: SocketAddr) -> Result<TcpSocket> {
    let socket = match local_addr {
        SocketAddr::V4(_addr) => {
            TcpSocket::new_v4().context("failed creating new v4 TCP socket")?
        }
        SocketAddr::V6(_addr) => {
            TcpSocket::new_v6().context("failed creating new v6 TCP socket")?
        }
    };

    #[cfg(not(target_os = "windows"))]
    socket
        .set_reuseport(true)
        .context("unable to set SO_REUSEPORT")?;
    #[cfg(target_os = "windows")]
    socket
        .set_reuseaddr(true)
        .context("unable to set SO_REUSEADDR")?;

    socket
        .bind(local_addr)
        .context("unable to bind local address")?;

    Ok(socket)
}
