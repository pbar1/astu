mod sqlite;

use anyhow::bail;
use anyhow::Result;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

pub use self::sqlite::SqliteDb;

#[async_trait]
#[enum_dispatch]
pub trait Db {
    // Required

    /// Saves an action result to the database.
    async fn save(&self, entry: &ResultEntry) -> Result<()>;

    /// Loads action results from the dataabse for a job.
    async fn load(&self, job_id: &str) -> Result<Vec<ResultEntry>>;

    // Defaults

    /// Migrates the database to the newest schema.
    ///
    /// By default this does nothing. Override this if needed.
    async fn migrate(&self) -> Result<()> {
        Ok(())
    }
}

#[enum_dispatch(Db)]
#[derive(Clone)]
pub enum DbImpl {
    Sqlite(SqliteDb),
}

impl DbImpl {
    /// # Errors
    ///
    /// If any db connection fails.
    pub async fn try_new(connection_string: &str) -> Result<Self> {
        if connection_string.contains("sqlite") || connection_string.contains(".db") {
            let db = SqliteDb::try_new(connection_string).await?;
            return Ok(db.into());
        }
        bail!("unable to build a db impl");
    }
}

/// Outcome of an action.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::FromRow)]
pub struct ResultEntry {
    pub job_id: String,
    pub target: String,
    pub error: Option<String>,
    pub exit_status: Option<u32>,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
}
