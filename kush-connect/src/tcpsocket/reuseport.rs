use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::SocketAddrV6;
use std::time::Duration;

use anyhow::Context;
use socket2::Domain;
use socket2::Protocol;
use socket2::SockAddr;
use socket2::Socket;
use socket2::Type;

pub struct ReuseportTcpFactory {
    reserved_v4: Socket,
    reserved_v6: Socket,
    addr_v4: SockAddr,
    addr_v6: SockAddr,
}

impl super::TcpFactory for ReuseportTcpFactory {
    fn connect_timeout(
        &self,
        addr: &SocketAddr,
        timeout: Duration,
    ) -> anyhow::Result<std::net::TcpStream> {
        let socket = self.get_reuseport_socket(addr)?;
        let remote_addr = SockAddr::from(addr.to_owned());
        socket.connect_timeout(&remote_addr, timeout)?;
        Ok(std::net::TcpStream::from(socket))
    }
}

#[async_trait::async_trait]
impl super::TcpFactoryAsync for ReuseportTcpFactory {
    async fn connect_timeout_async(
        &self,
        addr: &SocketAddr,
        timeout: Duration,
    ) -> anyhow::Result<tokio::net::TcpStream> {
        let socket = self.get_reuseport_socket_tokio(&addr)?;
        let connect = socket.connect(*addr);
        let stream = tokio::time::timeout(timeout, connect).await??;
        Ok(stream)
    }
}

impl ReuseportTcpFactory {
    pub fn try_new() -> anyhow::Result<Self> {
        let reserved_v4 = reserve_v4()?;
        let reserved_v6 = reserve_v6()?;

        let addr_v4 = reserved_v4.local_addr()?;
        let addr_v6 = reserved_v6.local_addr()?;

        Ok(Self {
            reserved_v4,
            reserved_v6,
            addr_v4,
            addr_v6,
        })
    }

    fn get_reuseport_socket(&self, remote_adr: &SocketAddr) -> anyhow::Result<Socket> {
        let socket = match remote_adr {
            SocketAddr::V4(_) => {
                let local_addr = self
                    .addr_v4
                    .as_socket_ipv4()
                    .context("unable to convert to std socketaddr")?;
                new_reuseport_socket_v4(*local_addr.ip(), local_addr.port())?
            }
            SocketAddr::V6(_) => {
                let local_addr = self
                    .addr_v6
                    .as_socket_ipv6()
                    .context("unable to convert to std socketaddr")?;
                new_reuseport_socket_v6(*local_addr.ip(), local_addr.port())?
            }
        };
        Ok(socket)
    }

    fn get_reuseport_socket_tokio(
        &self,
        remote_adr: &SocketAddr,
    ) -> anyhow::Result<tokio::net::TcpSocket> {
        let socket = match remote_adr {
            SocketAddr::V4(_) => {
                let local_addr = self
                    .addr_v4
                    .as_socket_ipv4()
                    .context("unable to convert to std socketaddr")?;
                let socket = tokio::net::TcpSocket::new_v4()?;
                socket.set_reuseport(true)?;
                socket.bind(local_addr.into())?;
                socket
            }
            SocketAddr::V6(_) => {
                let local_addr = self
                    .addr_v6
                    .as_socket_ipv6()
                    .context("unable to convert to std socketaddr")?;
                let socket = tokio::net::TcpSocket::new_v6()?;
                socket.set_reuseport(true)?;
                socket.bind(local_addr.into())?;
                socket
            }
        };
        Ok(socket)
    }
}

fn reserve_v4() -> anyhow::Result<Socket> {
    new_reuseport_socket_v4(Ipv4Addr::UNSPECIFIED, 0)
}

fn reserve_v6() -> anyhow::Result<Socket> {
    new_reuseport_socket_v6(Ipv6Addr::UNSPECIFIED, 0)
}

fn new_reuseport_socket_v4(ip: Ipv4Addr, port: u16) -> anyhow::Result<Socket> {
    let addr = SocketAddrV4::new(ip, port);
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_port(true)?;
    socket.bind(&addr.into())?;
    Ok(socket)
}

fn new_reuseport_socket_v6(ip: Ipv6Addr, port: u16) -> anyhow::Result<Socket> {
    let addr = SocketAddrV6::new(ip, port, 0, 0);
    let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_port(true)?;
    socket.bind(&addr.into())?;
    Ok(socket)
}

#[cfg(test)]
mod tests {
    use std::io::BufRead;
    use std::io::BufReader;
    use std::net::ToSocketAddrs;

    use rstest::rstest;

    use super::*;
    use crate::tcpsocket::TcpFactory;

    #[rstest]
    #[case("127.0.0.1:2222")]
    #[case("[::1]:2222")]
    fn works(#[case] input: &str) {
        let factory = ReuseportTcpFactory::try_new().unwrap();
        let remote = input.to_socket_addrs().unwrap().next().unwrap();
        let stream = factory
            .connect_timeout(&remote, Duration::from_secs(5))
            .unwrap();
        let mut reader = BufReader::new(stream);
        let mut output = String::new();
        reader.read_line(&mut output).unwrap();
        assert!(output.contains("SSH-2"));
    }
}
