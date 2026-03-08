//! Assortment of clients that can perform actions.

mod dynamic;
#[cfg(feature = "ssh")]
mod ssh;
mod tcp;

pub use dynamic::DynamicClientFactory;
#[cfg(feature = "ssh")]
pub use ssh::SshClient;
#[cfg(feature = "ssh")]
pub use ssh::SshClientFactory;
pub use tcp::TcpClient;
pub use tcp::TcpClientFactory;
