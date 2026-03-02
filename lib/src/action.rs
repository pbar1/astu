pub mod client;
pub mod transport;

use std::fmt;
use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use bstr::ByteSlice;
use enum_dispatch::enum_dispatch;

use crate::resolve::Target;

/// Actions that a client can perform.
#[async_trait]
#[enum_dispatch]
pub trait Client {
    /// Connect to a target.
    async fn connect(&mut self) -> Result<()>;

    /// Ping a target.
    async fn ping(&mut self) -> Result<Vec<u8>>;

    /// Authenticate with a target.
    async fn auth(&mut self, auth_type: &AuthPayload) -> Result<()>;

    /// Execute commands on a target.
    async fn exec(&mut self, command: &str, stdin: Option<ExecStdin>) -> Result<ExecOutput>;
}

/// All types of action clients.
#[enum_dispatch(Client)]
pub enum ClientImpl {
    Tcp(client::TcpClient),
    Ssh(client::SshClient),
    Local(client::LocalClient),
    Dummy(client::DummyClient),
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
    Local(client::LocalClientFactory),
    Dummy(client::DummyClientFactory),
}

/// Assortment of auth payloads that can be used with auth.
#[derive(Debug, Clone)]
pub enum AuthPayload {
    User(String),
    Password(String),
    SshKey(String),
    SshCert { key: String, cert: String },
    SshAgent { socket: String },
}

/// Output of a command run by exec.
#[derive(Clone)]
pub struct ExecOutput {
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum ExecStdin {
    Bytes(Vec<u8>),
    SpoolFile(PathBuf),
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
