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
        let (addr, user) = match target {
            Target::IpAddr(x) => (SocketAddr::new(x.into(), 22), None),
            Target::SocketAddr(x) => (x.into(), None),
            Target::Ssh { addr, user } => (addr, user),
            unsupported => bail!("unsupported ssh target: {unsupported}"),
        };
        Ok(SshClient::new(addr, self.tcp.clone(), user))
    }
}
