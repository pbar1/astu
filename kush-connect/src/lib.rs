pub mod ssh;
pub mod tcp;

#[async_trait::async_trait]
pub trait Ping {
    async fn ping(&self) -> anyhow::Result<String>;
}

pub struct ExecOutput {
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[async_trait::async_trait]
pub trait Exec {
    async fn exec(&mut self, command: &str) -> anyhow::Result<ExecOutput>;
}
