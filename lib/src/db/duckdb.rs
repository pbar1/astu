#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unused_async)]

mod compat;
mod gc;
mod queries;
mod schema;
mod writer;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Duration;
use chrono::Utc;
use duckdb::params;
use duckdb::Connection;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
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
    #[must_use]
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
    path: Arc<PathBuf>,
    sender: mpsc::Sender<WriteEvent>,
}

#[derive(Debug)]
enum WriteEvent {
    JobStart {
        job_id: Vec<u8>,
        started_at: String,
        command: String,
        concurrency: i64,
        task_count: i64,
    },
    JobEnd {
        job_id: Vec<u8>,
        finished_at: String,
    },
    TaskStart {
        task_id: Vec<u8>,
        job_id: Vec<u8>,
        started_at: String,
        target_uri: String,
        command: String,
    },
    TaskEnd {
        task_id: Vec<u8>,
        finished_at: String,
        status: String,
        exit_code: Option<i64>,
        error: Option<String>,
        connect_ms: i64,
        auth_ms: i64,
        exec_ms: i64,
    },
    TaskVar {
        task_id: Vec<u8>,
        key: String,
        value: String,
    },
    TaskLines {
        task_id: Vec<u8>,
        stream: String,
        lines: Vec<(i64, [u8; 16], String)>,
    },
    Sync {
        ack: oneshot::Sender<Result<()>>,
    },
}

impl DuckDb {
    pub async fn try_new(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let (sender, mut rx) = mpsc::channel::<WriteEvent>(16_384);

        let thread_path = path.clone();
        thread::spawn(move || {
            if let Err(error) = writer::run_writer_loop(&thread_path, &mut rx) {
                eprintln!("duckdb writer loop failed: {error:#}");
            }
        });

        let db = Self {
            path: Arc::new(path),
            sender,
        };
        db.sync().await?;
        Ok(db)
    }

    async fn sync(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(WriteEvent::Sync { ack: tx })
            .await
            .context("send sync event")?;
        rx.await.context("recv sync ack")?
    }

    fn read_conn(&self) -> Result<Connection> {
        Connection::open(&*self.path).context("open duckdb read connection")
    }

    pub async fn create_job(
        &self,
        job_id: &str,
        command: &str,
        concurrency: i64,
        task_count: i64,
    ) -> Result<()> {
        let job_blob = uuid_string_to_blob(job_id)?;
        self.sender
            .send(WriteEvent::JobStart {
                job_id: job_blob,
                started_at: Utc::now().to_rfc3339(),
                command: command.to_owned(),
                concurrency,
                task_count,
            })
            .await
            .context("send JobStart")
    }

    pub async fn finish_job(&self, job_id: &str) -> Result<()> {
        let job_blob = uuid_string_to_blob(job_id)?;
        self.sender
            .send(WriteEvent::JobEnd {
                job_id: job_blob,
                finished_at: Utc::now().to_rfc3339(),
            })
            .await
            .context("send JobEnd")?;
        self.sync().await
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
        self.sender
            .send(WriteEvent::TaskStart {
                task_id: task_blob,
                job_id: job_blob,
                started_at: Utc::now().to_rfc3339(),
                target_uri: target_uri.to_owned(),
                command: command.to_owned(),
            })
            .await
            .context("send TaskStart")
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
        self.sender
            .send(WriteEvent::TaskEnd {
                task_id: task_blob,
                finished_at: Utc::now().to_rfc3339(),
                status: status.as_str().to_owned(),
                exit_code,
                error: error.map(std::string::ToString::to_string),
                connect_ms,
                auth_ms,
                exec_ms,
            })
            .await
            .context("send TaskEnd")
    }

    pub async fn append_task_var(&self, task_id: &str, key: &str, value: &str) -> Result<()> {
        let task_blob = uuid_string_to_blob(task_id)?;
        self.sender
            .send(WriteEvent::TaskVar {
                task_id: task_blob,
                key: key.to_owned(),
                value: value.to_owned(),
            })
            .await
            .context("send TaskVar")
    }

    pub async fn append_stream_blob(
        &self,
        task_id: &str,
        stream: &str,
        bytes: &[u8],
    ) -> Result<()> {
        let task_blob = uuid_string_to_blob(task_id)?;
        let text = String::from_utf8_lossy(bytes);
        let mut lines = Vec::new();
        for (seq, line) in text.lines().enumerate() {
            let hash = xxh3_128(line.as_bytes()).to_be_bytes();
            lines.push((
                i64::try_from(seq).unwrap_or(i64::MAX),
                hash,
                line.to_owned(),
            ));
        }
        if lines.is_empty() {
            return Ok(());
        }
        self.sender
            .send(WriteEvent::TaskLines {
                task_id: task_blob,
                stream: stream.to_owned(),
                lines,
            })
            .await
            .context("send TaskLines")?;
        Ok(())
    }

    pub async fn last_job_id(&self) -> Result<Option<String>> {
        self.sync().await?;
        let conn = self.read_conn()?;
        let mut stmt = conn.prepare("SELECT value FROM meta WHERE key='last_job_id' LIMIT 1")?;
        let mut rows = stmt.query([])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let blob: Vec<u8> = row.get(0)?;
        Ok(Some(uuid_blob_to_string(&blob)?))
    }

    pub async fn jobs(&self, limit: i64) -> Result<Vec<JobRow>> {
        self.sync().await?;
        let conn = self.read_conn()?;
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
        self.sync().await?;
        let conn = self.read_conn()?;
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
        queries::freq(self, field, job_id, contains).await
    }

    pub async fn output(
        &self,
        field: DbField,
        job_id: &str,
        contains: Option<&str>,
        target_filter: Option<&str>,
    ) -> Result<Vec<OutputRow>> {
        queries::output(self, field, job_id, contains, target_filter).await
    }

    pub async fn trace(&self, job_id: &str, target_filter: Option<&str>) -> Result<Vec<TraceRow>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        let target = target_filter.unwrap_or("");
        self.sync().await?;
        let conn = self.read_conn()?;
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
        gc::gc_before(self, before).await
    }

    pub async fn command_for_job(&self, job_id: &str) -> Result<Option<String>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        self.sync().await?;
        let conn = self.read_conn()?;
        let mut stmt = conn.prepare("SELECT command FROM jobs WHERE job_id=? LIMIT 1")?;
        let mut rows = stmt.query(params![job_blob])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(row.get::<_, Option<String>>(0)?)
    }

    pub async fn canceled_tasks_for_job(
        &self,
        job_id: &str,
    ) -> Result<Vec<(String, String, String)>> {
        let job_blob = uuid_string_to_blob(job_id)?;
        self.sync().await?;
        let conn = self.read_conn()?;
        let mut stmt = conn.prepare(
            "SELECT task_id, target_uri, command FROM tasks WHERE job_id=? AND status='canceled' ORDER BY started_at ASC",
        )?;
        let mut rows = stmt.query(params![job_blob])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push((
                uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?,
                row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ));
        }
        Ok(out)
    }

    pub async fn task_vars_for_job(
        &self,
        job_id: &str,
    ) -> Result<HashMap<String, Vec<(String, String)>>> {
        let job_hex = job_id.replace('-', "").to_uppercase();
        self.sync().await?;
        let conn = self.read_conn()?;
        let mut stmt = conn.prepare(
            "SELECT tv.task_id, tv.key, tv.value
             FROM task_vars tv
             JOIN tasks t ON hex(t.task_id)=hex(tv.task_id)
             WHERE hex(t.job_id)=?
             ORDER BY tv.key ASC",
        )?;
        let mut rows = stmt.query(params![job_hex])?;
        let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        while let Some(row) = rows.next()? {
            let task_id = uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?;
            let key = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            let value = row.get::<_, Option<String>>(2)?.unwrap_or_default();
            map.entry(task_id).or_default().push((key, value));
        }
        Ok(map)
    }

    pub async fn task_vars_for_task(&self, task_id: &str) -> Result<Vec<(String, String)>> {
        let task_hex = task_id.replace('-', "").to_uppercase();
        self.sync().await?;
        let conn = self.read_conn()?;
        let mut stmt =
            conn.prepare("SELECT key, value FROM task_vars WHERE hex(task_id)=? ORDER BY key ASC")?;
        let mut rows = stmt.query(params![task_hex])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let key = row.get::<_, Option<String>>(0)?.unwrap_or_default();
            let value = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            out.push((key, value));
        }
        Ok(out)
    }
}

#[async_trait]
impl Db for DuckDb {
    async fn migrate(&self) -> Result<()> {
        self.sync().await
    }

    async fn save(&self, entry: &ResultEntry) -> Result<()> {
        compat::save_compat(self, entry).await
    }

    async fn load(&self, job_id: &str) -> Result<Vec<ResultEntry>> {
        compat::load_compat(self, job_id).await
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
