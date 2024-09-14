use anyhow::bail;
use anyhow::Result;
use astu_resolve::Target;

use crate::ssh::SshClient;
use crate::ssh::SshFactory;
use crate::tcp::TcpClient;
use crate::tcp::TcpClientFactory;

// Client ---------------------------------------------------------------------

/// [`Client`] contains variants that can perform actions.
pub enum Client {
    Tcp(TcpClient),
    Ssh(SshClient),
}

impl From<TcpClient> for Client {
    fn from(value: TcpClient) -> Self {
        Client::Tcp(value)
    }
}

impl From<SshClient> for Client {
    fn from(value: SshClient) -> Self {
        Client::Ssh(value)
    }
}

// ClientFactory --------------------------------------------------------------

/// Factory for mapping [`Target`] into [`Client`].
pub struct ClientFactory {
    tcp_factory: TcpClientFactory,
    ssh_factory: SshFactory,
}

impl ClientFactory {
    /// Maps [`Target`] into the default [`Client`] variant based on its type.
    ///
    /// For example,
    /// - [`Target::SocketAddr`] maps to [`Client::Tcp`]
    /// - [`Target::Ssh`] maps to [`Client::Ssh`]
    pub fn get_client(&self, target: Target) -> Result<Client> {
        let client = match target {
            Target::IpAddr(_) => self.tcp_factory.get_client(target)?.into(),
            Target::SocketAddr(_) => self.tcp_factory.get_client(target)?.into(),
            Target::Ssh { .. } => self.ssh_factory.get_client(target)?.into(),
            unsupported => bail!("no client supported for target: {unsupported}"),
        };
        Ok(client)
    }

    /// Maps [`Target`] into [`TcpClient`].
    pub fn get_tcp_client(&self, target: Target) -> Result<TcpClient> {
        self.tcp_factory.get_client(target)
    }

    /// Maps [`Target`] into [`SshClient`].
    pub fn get_ssh_client(&self, target: Target) -> Result<SshClient> {
        self.ssh_factory.get_client(target)
    }
}
