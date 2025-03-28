#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod file;
mod sqlite;

use anyhow::Result;

pub use crate::file::FileStore;
pub use crate::sqlite::SqliteDb;

#[async_trait::async_trait]
pub trait Db {
    async fn migrate(&self) -> Result<()>;

    async fn save_ping(&self, entry: &PingEntry) -> Result<()>;

    async fn load_ping(&self, job_id: &[u8]) -> Result<Vec<PingEntry>>;

    async fn save_exec(&self, entry: &ExecEntry) -> Result<()>;

    async fn load_exec(&self, job_id: &[u8]) -> Result<Vec<ExecEntry>>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::FromRow)]
pub struct PingEntry {
    pub job_id: Vec<u8>,
    pub target: String,
    pub error: Option<String>,
    pub message: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::FromRow)]
pub struct ExecEntry {
    pub job_id: Vec<u8>,
    pub target: String,
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
