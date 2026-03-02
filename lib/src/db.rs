mod duckdb;

use anyhow::Result;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

pub use self::duckdb::DbField;
pub use self::duckdb::DbTaskStatus;
pub use self::duckdb::DuckDb;
pub use self::duckdb::FreqRow;
pub use self::duckdb::JobRow;
pub use self::duckdb::OutputRow;
pub use self::duckdb::TaskRow;
pub use self::duckdb::TraceRow;

#[async_trait]
#[enum_dispatch]
pub trait Db {
    async fn save(&self, entry: &ResultEntry) -> Result<()>;
    async fn load(&self, job_id: &str) -> Result<Vec<ResultEntry>>;
    async fn migrate(&self) -> Result<()> {
        Ok(())
    }
}

#[enum_dispatch(Db)]
#[derive(Clone)]
pub enum DbImpl {
    Duck(DuckDb),
}

impl DbImpl {
    /// # Errors
    ///
    /// If db connection fails.
    pub async fn try_new(connection_string: &str) -> Result<Self> {
        Ok(DuckDb::try_new(connection_string).await?.into())
    }
}

/// Outcome of an action.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResultEntry {
    pub job_id: String,
    pub target: String,
    pub error: Option<String>,
    pub exit_status: Option<u32>,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
}
