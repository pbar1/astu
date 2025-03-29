//! Assortment of clients that can perform actions.

mod dynamic;
mod ssh;
mod tcp;

use anyhow::Result;
use astu_resolve::Target;
pub use dynamic::DynamicClientFactory;
pub use ssh::SshClient;
pub use ssh::SshClientFactory;
pub use tcp::TcpClient;
pub use tcp::TcpClientFactory;

/// Factory for building clients.
pub trait ClientFactory {
    fn client(&self, target: &Target) -> Option<Client>;
}

/// All types of actions clients.
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
