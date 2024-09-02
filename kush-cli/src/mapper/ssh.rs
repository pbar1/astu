use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::bail;
use kush_connect::ssh::SshClient;
use kush_connect::tcp::TcpFactoryAsync;
use kush_resolve::Target;

pub struct SshMapper {
    tcp: Arc<dyn TcpFactoryAsync + Send + Sync>,
}

impl SshMapper {
    pub fn new(tcp: Arc<dyn TcpFactoryAsync + Send + Sync>) -> Self {
        Self { tcp }
    }

    pub fn get_client(&self, target: Target) -> anyhow::Result<SshClient> {
        let addr = match target {
            Target::Ipv4Addr(x) => SocketAddr::new(x.into(), 22),
            Target::Ipv6Addr(x) => SocketAddr::new(x.into(), 22),
            Target::SocketAddrV4(x) => x.into(),
            Target::SocketAddrV6(x) => x.into(),
            unsupported => bail!("unsupported ssh target: {unsupported}"),
        };
        Ok(SshClient::new(addr, self.tcp.clone()))
    }
}
