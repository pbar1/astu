use std::str::FromStr;

use anyhow::Result;
use futures::StreamExt;
use sqlx::migrate;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::error;

use crate::Db;
use crate::ExecEntry;

pub struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDb {
    pub async fn try_new(url: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(opts).await?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl Db for SqliteDb {
    async fn migrate(&self) -> Result<()> {
        migrate!()
            .run(&self.pool)
            .await
            .map_err(anyhow::Error::from)
    }

    async fn save_exec(&self, entry: &ExecEntry) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO exec_entries (job_id, target, exit_status, stdout, stderr) VALUES (?, ?, ?, ?, ?)"#,
        ).bind(&entry.job_id).bind(&entry.target).bind(&entry.exit_status).bind(&entry.stdout).bind(&entry.stderr).execute(&self.pool).await?;
        Ok(())
    }

    async fn load_exec(&self, job_id: &[u8]) -> Result<Vec<ExecEntry>> {
        let mut stream =
            sqlx::query_as::<_, ExecEntry>(r#"SELECT * FROM exec_entries WHERE job_id = ?"#)
                .bind(job_id)
                .fetch(&self.pool);

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
        db.migrate().await.unwrap();

        let entry_foo = ExecEntry {
            job_id: b"0".into(),
            target: "foo".into(),
            exit_status: 0,
            stdout: b"foo_stdout".into(),
            stderr: b"foo_stderr".into(),
        };
        let entry_bar = ExecEntry {
            job_id: b"0".into(),
            target: "bar".into(),
            exit_status: 1,
            stdout: b"bar_stdout".into(),
            stderr: b"bar_stderr".into(),
        };
        let entry_baz = ExecEntry {
            job_id: b"1".into(),
            target: "baz".into(),
            exit_status: 2,
            stdout: b"baz_stdout".into(),
            stderr: b"baz_stderr".into(),
        };

        db.save_exec(&entry_foo).await.unwrap();
        db.save_exec(&entry_bar).await.unwrap();
        db.save_exec(&entry_baz).await.unwrap();

        let entries = db.load_exec(b"0").await.unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry_foo);
        assert_eq!(entries[1], entry_bar);
    }
}
