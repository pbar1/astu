use crate::ssh::SshClient;
use crate::tcp::TcpClient;

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
