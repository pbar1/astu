#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod transport;

use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bstr::ByteSlice;
use client::ClientFactory;
use transport::TransportFactory;

// Trait ----------------------------------------------------------------------

/// Connect to a target.
#[async_trait]
pub trait Connect {
    async fn connect(&mut self) -> Result<()>;
}

/// Ping a target.
#[async_trait]
pub trait Ping {
    async fn ping(self) -> Result<String>;
}

/// Authenticate with a target.
#[async_trait]
pub trait Auth {
    async fn auth(&mut self, auth_type: &AuthType) -> Result<()>;
}

/// Execute commands on a target.
#[async_trait]
pub trait Exec {
    async fn exec(&mut self, command: &str) -> Result<ExecOutput>;
}

// Data types -----------------------------------------------------------------

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
