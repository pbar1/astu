mod default;
mod reuseport;

use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use anyhow::Result;

pub trait TcpFactory {
    fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> Result<TcpStream>;
}
