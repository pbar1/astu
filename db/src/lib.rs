#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod sqlite;

use anyhow::Result;

pub use crate::sqlite::SqliteDb;

#[async_trait::async_trait]
pub trait Db {
    async fn migrate(&self) -> Result<()>;

    async fn save_exec(&self, entry: &ExecEntry) -> Result<()>;

    async fn load_exec(&self, job_id: &[u8]) -> Result<Vec<ExecEntry>>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::FromRow)]
pub struct ExecEntry {
    pub job_id: Vec<u8>,
    pub target: String,
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
