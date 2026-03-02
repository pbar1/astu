//! Assortment of clients that can perform actions.

mod dummy;
mod dynamic;
mod local;
mod ssh;
mod tcp;

pub use dummy::DummyClient;
pub use dummy::DummyClientFactory;
pub use dynamic::DynamicClientFactory;
pub use local::LocalClient;
pub use local::LocalClientFactory;
pub use ssh::SshClient;
pub use ssh::SshClientFactory;
pub use tcp::TcpClient;
pub use tcp::TcpClientFactory;
