use std::str::FromStr;

use anyhow::Result;
use bon::Builder;
use futures::StreamExt;
use sqlx::migrate;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::error;

use super::Db;
use super::ResultEntry;

/// `SQLite` persistence layer.
#[derive(Debug, Clone, Builder)]
pub struct SqliteDb {
    #[builder(into)]
    url: String,
    #[builder(skip)]
    pool: Option<SqlitePool>,
}

impl SqliteDb {
    /// Returns a connection to the database with migrations having been run.
    ///
    /// # Errors
    ///
    /// - If connect fails
    /// - If schema migration fails
    pub async fn try_new(url: &str) -> Result<Self> {
        let db = Self::builder().url(url).build().connect().await?;
        db.migrate().await?;
        Ok(db)
    }

    /// Connect to the database.
    ///
    /// # Errors
    ///
    /// - If connect fails
    pub async fn connect(mut self) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(&self.url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(opts).await?;
        self.pool = Some(pool);
        Ok(self)
    }

    /// Gets the pool. Panics [`SqliteDb::connect`] has not yet been called!
    fn pool(&self) -> SqlitePool {
        self.pool.clone().expect("SqlitePool not yet connected")
    }
}

#[async_trait::async_trait]
impl Db for SqliteDb {
    async fn migrate(&self) -> Result<()> {
        migrate!("migrations/sqlite")
            .run(&self.pool())
            .await
            .map_err(anyhow::Error::from)
    }

    async fn save(&self, entry: &ResultEntry) -> Result<()> {
        sqlx::query(
            r"INSERT INTO results (job_id, target, error, exit_status, stdout, stderr) VALUES (?, ?, ?, ?, ?, ?)",
        )
            .bind(&entry.job_id)
            .bind(&entry.target)
            .bind(&entry.error)
            .bind(entry.exit_status)
            .bind(&entry.stdout)
            .bind(&entry.stderr)
            .execute(&self.pool())
            .await?;
        Ok(())
    }

    async fn load(&self, job_id: &str) -> Result<Vec<ResultEntry>> {
        let pool = self.pool();

        let mut stream =
            sqlx::query_as::<_, ResultEntry>(r"SELECT * FROM results WHERE job_id = ?")
                .bind(job_id)
                .fetch(&pool);

        let mut entries = Vec::new();
        while let Some(entry) = stream.next().await {
            match entry {
                Ok(e) => entries.push(e),
                Err(error) => error!(?error, "row error in load_exec"),
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn save_load_works() {
        let db = SqliteDb::try_new("sqlite::memory:").await.unwrap();

        let entry_foo = ResultEntry {
            job_id: "0".into(),
            target: "foo".into(),
            error: None,
            exit_status: Some(0),
            stdout: Some(b"foo_stdout".into()),
            stderr: Some(b"foo_stderr".into()),
        };
        let entry_bar = ResultEntry {
            job_id: "0".into(),
            target: "bar".into(),
            error: None,
            exit_status: Some(1),
            stdout: Some(b"bar_stdout".into()),
            stderr: Some(b"bar_stderr".into()),
        };
        let entry_quux = ResultEntry {
            job_id: "1".into(),
            target: "quux".into(),
            error: None,
            exit_status: Some(2),
            stdout: Some(b"quux_stdout".into()),
            stderr: Some(b"quux_stderr".into()),
        };

        db.save(&entry_foo).await.unwrap();
        db.save(&entry_bar).await.unwrap();
        db.save(&entry_quux).await.unwrap();

        let entries = db.load("0").await.unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry_foo);
        assert_eq!(entries[1], entry_bar);
    }
}
