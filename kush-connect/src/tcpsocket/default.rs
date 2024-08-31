use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use anyhow::Result;

pub struct DefaultTcpFactory;

impl super::TcpFactory for DefaultTcpFactory {
    fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> Result<TcpStream> {
        let stream = TcpStream::connect_timeout(addr.into(), timeout)?;
        Ok(stream)
    }
}
