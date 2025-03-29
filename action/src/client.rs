//! Assortment of clients that can perform actions.

mod dynamic;
mod ssh;
mod tcp;

pub use dynamic::DynamicClientFactory;
pub use ssh::SshClient;
pub use ssh::SshClientFactory;
pub use tcp::TcpClient;
pub use tcp::TcpClientFactory;
