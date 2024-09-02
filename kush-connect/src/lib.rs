pub mod ssh;
pub mod tcp;

use std::time::Duration;

use ssh_key::Certificate;

#[async_trait::async_trait]
pub trait Connect {
    async fn connect(&mut self, timeout: Duration) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait Auth {
    async fn auth(&mut self, auth_type: &AuthType) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait Ping {
    async fn ping(&self) -> anyhow::Result<String>;
}

#[async_trait::async_trait]
pub trait Exec {
    async fn exec(&mut self, command: &str) -> anyhow::Result<ExecOutput>;
}

pub enum AuthType {
    User(String),
    Password(String),
    SshKey(String),
    SshCert { key: String, cert: Certificate },
    SshAgent { socket: String },
}

pub struct ExecOutput {
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
