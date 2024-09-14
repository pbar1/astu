#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod ssh;
pub mod tcp;

use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Connect {
    async fn connect(&mut self, timeout: Duration) -> Result<()>;
}

#[async_trait]
pub trait Ping {
    async fn ping(self) -> Result<String>;
}

#[async_trait]
pub trait Auth {
    async fn auth(&mut self, auth_type: &AuthType) -> Result<()>;
}

#[async_trait]
pub trait Exec {
    async fn exec(&mut self, command: &str) -> Result<ExecOutput>;
}

#[derive(Debug, Clone)]
pub enum AuthType {
    User(String),
    Password(String),
    SshKey(String),
    SshCert {
        key: String,
        cert: ssh_key::Certificate,
    },
    SshAgent {
        socket: String,
    },
}

pub struct ExecOutput {
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
