use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Duration;
use chrono::Utc;
use duckdb::Connection;
use duckdb::params;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_128;

use super::Db;
use super::ResultEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbTaskStatus {
    Complete,
    Failed,
    Canceled,
}

impl DbTaskStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DbField {
    Stdout,
    Stderr,
    Exitcode,
    Error,
}

#[derive(Debug, Clone)]
pub struct JobRow {
    pub job_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub command: String,
    pub task_count: i64,
}

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub task_id: String,
    pub target: String,
    pub status: String,
    pub command: String,
    pub exit_code: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct FreqRow {
    pub value: String,
    pub count: i64,
}

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub task_id: String,
    pub target: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct TraceRow {
    pub task_id: String,
    pub target: String,
    pub status: String,
    pub error: String,
    pub connect_ms: i64,
    pub auth_ms: i64,
    pub exec_ms: i64,
}

#[derive(Debug, Clone)]
pub struct DuckDb {
    conn: Arc<Mutex<Connection>>,
}

impl DuckDb {
    pub async fn try_new(path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path).context("open duckdb")?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.migrate().await?;
        Ok(db)
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("duckdb mutex poisoned")
    }

    pub async fn create_job(
        &self,
        job_id: &str,
        command: &str,
        concurrency: i64,
        task_count: i64,
    ) -> Result<()> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let now = Utc::now().to_rfc3339();
        let conn = self.conn();
        conn.execute(
            "INSERT INTO jobs(job_id, started_at, command, concurrency, task_count) VALUES (?, ?, ?, ?, ?)",
            params![job_blob, now, command, concurrency, task_count],
        )?;
        Ok(())
    }

    pub async fn finish_job(&self, job_id: &str) -> Result<()> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let now = Utc::now().to_rfc3339();
        let conn = self.conn();
        conn.execute(
            "UPDATE jobs SET finished_at = ? WHERE job_id = ?",
            params![now, job_blob],
        )?;
        conn.execute(
            "INSERT INTO meta(key, value) VALUES ('last_job_id', ?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![job_blob],
        )?;
        Ok(())
    }

    pub async fn create_task(
        &self,
        task_id: &str,
        job_id: &str,
        target_uri: &str,
        command: &str,
    ) -> Result<()> {
        let task_blob = uuid_string_to_blob(task_id)?;
        let job_blob = uuid_string_to_blob(job_id)?;
        let now = Utc::now().to_rfc3339();
        let conn = self.conn();
        conn.execute(
            "INSERT INTO tasks(task_id, job_id, started_at, target_uri, command, status) VALUES (?, ?, ?, ?, ?, ?)",
            params![task_blob, job_blob, now, target_uri, command, DbTaskStatus::Canceled.as_str()],
        )?;
        Ok(())
    }

    pub async fn finish_task(
        &self,
        task_id: &str,
        status: DbTaskStatus,
        exit_code: Option<i64>,
        error: Option<&str>,
        connect_ms: i64,
        auth_ms: i64,
        exec_ms: i64,
    ) -> Result<()> {
        let task_blob = uuid_string_to_blob(task_id)?;
        let now = Utc::now().to_rfc3339();
        let conn = self.conn();
        conn.execute(
            "UPDATE tasks SET finished_at=?, exit_code=?, error=?, status=?, connect_ms=?, auth_ms=?, exec_ms=? WHERE task_id=?",
            params![now, exit_code, error, status.as_str(), connect_ms, auth_ms, exec_ms, task_blob],
        )?;
        Ok(())
    }

    pub async fn append_task_var(&self, task_id: &str, key: &str, value: &str) -> Result<()> {
        let task_blob = uuid_string_to_blob(task_id)?;
        let conn = self.conn();
        conn.execute(
            "INSERT INTO task_vars(task_id, key, value) VALUES (?, ?, ?) ON CONFLICT(task_id, key) DO UPDATE SET value=excluded.value",
            params![task_blob, key, value],
        )?;
        Ok(())
    }

    pub async fn append_stream_blob(&self, task_id: &str, stream: &str, bytes: &[u8]) -> Result<()> {
        let task_blob = uuid_string_to_blob(task_id)?;
        let text = String::from_utf8_lossy(bytes);
        let conn = self.conn();

        for (seq, line) in text.lines().enumerate() {
            let hash = xxh3_128(line.as_bytes()).to_be_bytes().to_vec();
            conn.execute(
                "INSERT INTO line_dict(line_hash, line_text) VALUES (?, ?) ON CONFLICT(line_hash) DO NOTHING",
                params![hash.clone(), line],
            )?;
            conn.execute(
                "INSERT INTO task_lines(task_id, stream, seq, line_hash) VALUES (?, ?, ?, ?)",
                params![task_blob.clone(), stream, i64::try_from(seq).unwrap_or(i64::MAX), hash],
            )?;
        }

        Ok(())
    }

    pub async fn last_job_id(&self) -> Result<Option<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT value FROM meta WHERE key='last_job_id' LIMIT 1")?;
        let mut rows = stmt.query([])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let blob: Vec<u8> = row.get(0)?;
        Ok(Some(uuid_blob_to_string(&blob)?))
    }

    pub async fn jobs(&self, limit: i64) -> Result<Vec<JobRow>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT job_id, CAST(started_at AS VARCHAR), CAST(finished_at AS VARCHAR), command, task_count FROM jobs ORDER BY started_at DESC LIMIT ?",
        )?;
        let mut rows = stmt.query(params![limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(JobRow {
                job_id: uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?,
                started_at: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                finished_at: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                command: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                task_count: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
            });
        }
        Ok(out)
    }

    pub async fn tasks(&self, job_id: &str) -> Result<Vec<TaskRow>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT task_id, target_uri, status, command, exit_code FROM tasks WHERE job_id=? ORDER BY started_at ASC",
        )?;
        let mut rows = stmt.query(params![job_blob])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(TaskRow {
                task_id: uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?,
                target: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                status: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                command: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                exit_code: row.get::<_, Option<i64>>(4)?,
            });
        }
        Ok(out)
    }

    pub async fn freq(
        &self,
        field: DbField,
        job_id: &str,
        contains: Option<&str>,
    ) -> Result<Vec<FreqRow>> {
        let limit = 200_i64;
        let job_blob = uuid_string_to_blob(job_id)?;
        let conn = self.conn();
        let contains = contains.unwrap_or("");

        let sql = match field {
            DbField::Stdout => {
                "WITH assembled AS (
                    SELECT tl.task_id, string_agg(ld.line_text, '\n' ORDER BY tl.seq) AS value
                    FROM task_lines tl
                    JOIN tasks t ON t.task_id = tl.task_id
                    JOIN line_dict ld ON ld.line_hash = tl.line_hash
                    WHERE tl.stream = 'stdout' AND t.job_id = ?
                    GROUP BY tl.task_id
                )
                SELECT value, COUNT(*) AS count
                FROM assembled
                WHERE (? = '' OR value LIKE '%' || ? || '%')
                GROUP BY value
                ORDER BY count DESC, value ASC
                LIMIT ?"
            }
            DbField::Stderr => {
                "WITH assembled AS (
                    SELECT tl.task_id, string_agg(ld.line_text, '\n' ORDER BY tl.seq) AS value
                    FROM task_lines tl
                    JOIN tasks t ON t.task_id = tl.task_id
                    JOIN line_dict ld ON ld.line_hash = tl.line_hash
                    WHERE tl.stream = 'stderr' AND t.job_id = ?
                    GROUP BY tl.task_id
                )
                SELECT value, COUNT(*) AS count
                FROM assembled
                WHERE (? = '' OR value LIKE '%' || ? || '%')
                GROUP BY value
                ORDER BY count DESC, value ASC
                LIMIT ?"
            }
            DbField::Error => {
                "SELECT error as value, COUNT(*) as count
                 FROM tasks
                 WHERE job_id=? AND error IS NOT NULL AND error <> ''
                   AND (? = '' OR error LIKE '%' || ? || '%')
                 GROUP BY value
                 ORDER BY count DESC, value ASC
                 LIMIT ?"
            }
            DbField::Exitcode => {
                "SELECT CAST(COALESCE(exit_code, -1) AS VARCHAR) as value, COUNT(*) as count
                 FROM tasks
                 WHERE job_id=?
                   AND (? = '' OR CAST(COALESCE(exit_code, -1) AS VARCHAR) LIKE '%' || ? || '%')
                 GROUP BY value
                 ORDER BY count DESC, value ASC
                 LIMIT ?"
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query(params![job_blob, contains, contains, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(FreqRow {
                value: row.get(0)?,
                count: row.get(1)?,
            });
        }
        Ok(out)
    }

    pub async fn output(
        &self,
        field: DbField,
        job_id: &str,
        contains: Option<&str>,
        target_filter: Option<&str>,
    ) -> Result<Vec<OutputRow>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let contains = contains.unwrap_or("");
        let target = target_filter.unwrap_or("");
        let conn = self.conn();

        let sql = match field {
            DbField::Stdout => {
                "WITH assembled AS (
                    SELECT tl.task_id, string_agg(ld.line_text, '\n' ORDER BY tl.seq) AS value
                    FROM task_lines tl
                    JOIN tasks t ON t.task_id = tl.task_id
                    JOIN line_dict ld ON ld.line_hash = tl.line_hash
                    WHERE tl.stream='stdout' AND t.job_id=?
                    GROUP BY tl.task_id
                )
                SELECT t.task_id, t.target_uri, a.value
                FROM assembled a JOIN tasks t ON t.task_id=a.task_id
                WHERE (? = '' OR a.value LIKE '%' || ? || '%')
                  AND (? = '' OR t.target_uri = ?)
                ORDER BY t.started_at ASC"
            }
            DbField::Stderr => {
                "WITH assembled AS (
                    SELECT tl.task_id, string_agg(ld.line_text, '\n' ORDER BY tl.seq) AS value
                    FROM task_lines tl
                    JOIN tasks t ON t.task_id = tl.task_id
                    JOIN line_dict ld ON ld.line_hash = tl.line_hash
                    WHERE tl.stream='stderr' AND t.job_id=?
                    GROUP BY tl.task_id
                )
                SELECT t.task_id, t.target_uri, a.value
                FROM assembled a JOIN tasks t ON t.task_id=a.task_id
                WHERE (? = '' OR a.value LIKE '%' || ? || '%')
                  AND (? = '' OR t.target_uri = ?)
                ORDER BY t.started_at ASC"
            }
            DbField::Error => {
                "SELECT task_id, target_uri, error
                 FROM tasks
                 WHERE job_id=? AND error IS NOT NULL AND error <> ''
                   AND (? = '' OR error LIKE '%' || ? || '%')
                   AND (? = '' OR target_uri = ?)
                 ORDER BY started_at ASC"
            }
            DbField::Exitcode => {
                "SELECT task_id, target_uri, CAST(COALESCE(exit_code, -1) AS VARCHAR)
                 FROM tasks
                 WHERE job_id=?
                   AND (? = '' OR CAST(COALESCE(exit_code, -1) AS VARCHAR) LIKE '%' || ? || '%')
                   AND (? = '' OR target_uri = ?)
                 ORDER BY started_at ASC"
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query(params![job_blob, contains, contains, target, target])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(OutputRow {
                task_id: uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?,
                target: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                value: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            });
        }
        Ok(out)
    }

    pub async fn trace(&self, job_id: &str, target_filter: Option<&str>) -> Result<Vec<TraceRow>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let target = target_filter.unwrap_or("");
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT task_id, target_uri, status, COALESCE(error,''), COALESCE(connect_ms,0), COALESCE(auth_ms,0), COALESCE(exec_ms,0)
             FROM tasks
             WHERE job_id=? AND (? = '' OR target_uri = ?)
             ORDER BY started_at ASC",
        )?;
        let mut rows = stmt.query(params![job_blob, target, target])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(TraceRow {
                task_id: uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?,
                target: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                status: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                error: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                connect_ms: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                auth_ms: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                exec_ms: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
            });
        }
        Ok(out)
    }

    pub async fn gc_before(&self, before: Duration) -> Result<()> {
        let cutoff = (Utc::now() - before).to_rfc3339();
        let conn = self.conn();

        let mut old_jobs = Vec::<Vec<u8>>::new();
        let mut stmt = conn.prepare("SELECT job_id FROM jobs WHERE started_at <= ?")?;
        let mut rows = stmt.query(params![cutoff])?;
        while let Some(row) = rows.next()? {
            old_jobs.push(row.get(0)?);
        }

        for job_blob in old_jobs {
            conn.execute("DELETE FROM task_lines WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id=?)", params![job_blob.clone()])?;
            conn.execute("DELETE FROM task_vars WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id=?)", params![job_blob.clone()])?;
            conn.execute("DELETE FROM tasks WHERE job_id=?", params![job_blob.clone()])?;
            conn.execute("DELETE FROM jobs WHERE job_id=?", params![job_blob])?;
        }
        Ok(())
    }

    pub async fn command_for_job(&self, job_id: &str) -> Result<Option<String>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT command FROM jobs WHERE job_id=? LIMIT 1")?;
        let mut rows = stmt.query(params![job_blob])?;
        let Some(row) = rows.next()? else { return Ok(None) };
        Ok(row.get::<_, Option<String>>(0)?)
    }

    pub async fn canceled_tasks_for_job(&self, job_id: &str) -> Result<Vec<(String, String)>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT task_id, target_uri FROM tasks WHERE job_id=? AND status='canceled' ORDER BY started_at ASC",
        )?;
        let mut rows = stmt.query(params![job_blob])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push((
                uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?,
                row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            ));
        }
        Ok(out)
    }

    pub async fn task_vars_for_job(&self, job_id: &str) -> Result<HashMap<String, Vec<(String, String)>>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT tv.task_id, tv.key, tv.value FROM task_vars tv JOIN tasks t ON t.task_id=tv.task_id WHERE t.job_id=?",
        )?;
        let mut rows = stmt.query(params![job_blob])?;
        let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        while let Some(row) = rows.next()? {
            let task_id = uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?;
            let key = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            let value = row.get::<_, Option<String>>(2)?.unwrap_or_default();
            map.entry(task_id).or_default().push((key, value));
        }
        Ok(map)
    }
}

#[async_trait]
impl Db for DuckDb {
    async fn migrate(&self) -> Result<()> {
        let conn = self.conn();
        conn.execute_batch(
            "
CREATE TABLE IF NOT EXISTS jobs (
  job_id BLOB PRIMARY KEY,
  started_at TIMESTAMP,
  finished_at TIMESTAMP,
  command TEXT,
  concurrency BIGINT,
  task_count BIGINT
);
CREATE TABLE IF NOT EXISTS tasks (
  task_id BLOB PRIMARY KEY,
  job_id BLOB,
  started_at TIMESTAMP,
  finished_at TIMESTAMP,
  target_uri TEXT,
  command TEXT,
  status TEXT,
  exit_code BIGINT,
  error TEXT,
  connect_ms BIGINT,
  auth_ms BIGINT,
  exec_ms BIGINT
);
CREATE TABLE IF NOT EXISTS task_vars (
  task_id BLOB,
  key TEXT,
  value TEXT,
  PRIMARY KEY(task_id, key)
);
CREATE TABLE IF NOT EXISTS task_lines (
  task_id BLOB,
  stream TEXT,
  seq BIGINT,
  line_hash BLOB,
  PRIMARY KEY(task_id, stream, seq)
);
CREATE TABLE IF NOT EXISTS line_dict (
  line_hash BLOB PRIMARY KEY,
  line_text TEXT
);
CREATE TABLE IF NOT EXISTS meta (
  key TEXT PRIMARY KEY,
  value BLOB
);
",
        )?;
        Ok(())
    }

    async fn save(&self, entry: &ResultEntry) -> Result<()> {
        // Compatibility path: map legacy result rows into tasks table.
        let task_id = Uuid::now_v7().hyphenated().to_string();
        self.create_task(&task_id, &entry.job_id, &entry.target, "").await?;

        if let Some(stdout) = &entry.stdout {
            self.append_stream_blob(&task_id, "stdout", stdout).await?;
        }
        if let Some(stderr) = &entry.stderr {
            self.append_stream_blob(&task_id, "stderr", stderr).await?;
        }

        let status = if entry.error.is_some() || entry.exit_status.unwrap_or(0) != 0 {
            DbTaskStatus::Failed
        } else {
            DbTaskStatus::Complete
        };

        self.finish_task(
            &task_id,
            status,
            entry.exit_status.map(i64::from),
            entry.error.as_deref(),
            0,
            0,
            0,
        )
        .await
    }

    async fn load(&self, job_id: &str) -> Result<Vec<ResultEntry>> {
        let rows = self.output(DbField::Stdout, job_id, None, None).await?;
        let stderr_map = self
            .output(DbField::Stderr, job_id, None, None)
            .await?
            .into_iter()
            .map(|x| (x.task_id, x.value))
            .collect::<HashMap<_, _>>();
        let exit_map = self
            .output(DbField::Exitcode, job_id, None, None)
            .await?
            .into_iter()
            .map(|x| (x.task_id, x.value))
            .collect::<HashMap<_, _>>();
        let err_map = self
            .output(DbField::Error, job_id, None, None)
            .await?
            .into_iter()
            .map(|x| (x.task_id, x.value))
            .collect::<HashMap<_, _>>();

        let mut out = Vec::new();
        for row in rows {
            let exit_status = exit_map.get(&row.task_id).and_then(|x| x.parse::<u32>().ok());
            out.push(ResultEntry {
                job_id: job_id.to_owned(),
                target: row.target,
                error: err_map.get(&row.task_id).cloned(),
                exit_status,
                stdout: Some(row.value.into_bytes()),
                stderr: stderr_map.get(&row.task_id).map(|x| x.clone().into_bytes()),
            });
        }
        Ok(out)
    }
}

fn uuid_string_to_blob(value: &str) -> Result<Vec<u8>> {
    let parsed = Uuid::parse_str(value).with_context(|| format!("invalid uuid: {value}"))?;
    Ok(parsed.as_bytes().to_vec())
}

fn uuid_blob_to_string(value: &[u8]) -> Result<String> {
    let uuid = Uuid::from_slice(value)
        .with_context(|| format!("invalid uuid blob length {}", value.len()))?;
    Ok(uuid.hyphenated().to_string())
}
