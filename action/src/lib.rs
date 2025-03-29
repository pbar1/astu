#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod transport;

use std::fmt;

use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;
use bstr::ByteSlice;
use enum_dispatch::enum_dispatch;

/// Actions that a client can perform.
#[async_trait]
#[enum_dispatch]
pub trait Client {
    /// Connect to a target
    async fn connect(&mut self) -> Result<()>;

    /// Authenticate with a target.
    async fn auth(&mut self, auth_type: &AuthType) -> Result<()>;

    /// Execute commands on a target.
    async fn exec(&mut self, command: &str) -> Result<ExecOutput>;
}

/// All types of action clients.
#[enum_dispatch(Client)]
pub enum ClientImpl {
    Tcp(client::TcpClient),
    Ssh(client::SshClient),
}

/// Factory for building clients.
#[enum_dispatch]
pub trait ClientFactory {
    fn client(&self, target: &Target) -> Option<ClientImpl>;
}

/// All types of action client factories.
#[enum_dispatch(ClientFactory)]
#[derive(Debug, Clone)]
pub enum ClientFactoryImpl {
    Tcp(client::TcpClientFactory),
    Ssh(client::SshClientFactory),
}

/// Assortment of auth payloads that can be used with [`Auth`].
#[derive(Debug, Clone)]
pub enum AuthType {
    User(String),
    Password(String),
    SshKey(String),
    SshCert { key: String, cert: String },
    SshAgent { socket: String },
}

/// Output of a command run by [`Exec`].
#[derive(Clone)]
pub struct ExecOutput {
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl fmt::Debug for ExecOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecOutput")
            .field("exit_status", &self.exit_status)
            .field("stdout", &self.stdout.as_bstr())
            .field("stderr", &self.stderr.as_bstr())
            .finish()
    }
}
