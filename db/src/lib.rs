#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod sqlite;

use anyhow::Result;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

pub use crate::sqlite::SqliteDb;

#[async_trait]
#[enum_dispatch]
pub trait Db {
    // Required

    /// Saves a [`PingEntry`] to the database.
    async fn save_ping(&self, entry: &PingEntry) -> Result<()>;

    /// Loads a [`PingEntry`] from the database by `job_id`.
    async fn load_ping(&self, job_id: &[u8]) -> Result<Vec<PingEntry>>;

    /// Saves an [`ExecEntry`] to the database.
    async fn save_exec(&self, entry: &ExecEntry) -> Result<()>;

    /// Loads an [`ExecEntry`] from the database by `job_id`.
    async fn load_exec(&self, job_id: &[u8]) -> Result<Vec<ExecEntry>>;

    // Defaults

    /// Migrates the database to the newest schema.
    ///
    /// By default this does nothing. Override this if needed.
    async fn migrate(&self) -> Result<()> {
        Ok(())
    }
}

#[enum_dispatch(Db)]
pub enum DbImpl {
    Sqlite(SqliteDb),
}

/// Outcome of a `ping` run.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::FromRow)]
pub struct PingEntry {
    pub job_id: Vec<u8>,
    pub target: String,
    pub error: Option<String>,
    pub message: Vec<u8>,
}

/// Outcome of an `exec` run.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::FromRow)]
pub struct ExecEntry {
    pub job_id: Vec<u8>,
    pub target: String,
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
