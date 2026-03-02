#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unused_async)]

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use anyhow::anyhow;
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
struct PendingJob {
    started_at: String,
    command: String,
    concurrency: i64,
    task_count: i64,
}

#[derive(Debug)]
struct PendingTask {
    job_id: Vec<u8>,
    started_at: String,
    target_uri: String,
    command: String,
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
            if let Err(error) = run_writer_loop(&thread_path, &mut rx) {
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
        let limit = 200_i64;
        let job_blob = uuid_string_to_blob(job_id)?;
        self.sync().await?;
        let conn = self.read_conn()?;
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
        self.sync().await?;
        let conn = self.read_conn()?;

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
        self.sync().await?;
        let cutoff = (Utc::now() - before).to_rfc3339();
        let conn = self.read_conn()?;

        let mut old_jobs = Vec::<Vec<u8>>::new();
        let mut stmt = conn.prepare("SELECT job_id FROM jobs WHERE started_at <= ?")?;
        let mut rows = stmt.query(params![cutoff])?;
        while let Some(row) = rows.next()? {
            old_jobs.push(row.get(0)?);
        }

        for job_blob in old_jobs {
            conn.execute("DELETE FROM task_lines WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id=?)", params![job_blob.clone()])?;
            conn.execute(
                "DELETE FROM task_vars WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id=?)",
                params![job_blob.clone()],
            )?;
            conn.execute(
                "DELETE FROM tasks WHERE job_id=?",
                params![job_blob.clone()],
            )?;
            conn.execute("DELETE FROM jobs WHERE job_id=?", params![job_blob])?;
        }
        Ok(())
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
        let job_blob = uuid_string_to_blob(job_id)?;
        self.sync().await?;
        let conn = self.read_conn()?;
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

fn run_writer_loop(path: &Path, rx: &mut mpsc::Receiver<WriteEvent>) -> Result<()> {
    let conn = Connection::open(path).context("open duckdb")?;
    init_schema(&conn)?;

    let mut jobs = conn
        .appender_with_columns(
            "jobs",
            &[
                "job_id",
                "started_at",
                "finished_at",
                "command",
                "concurrency",
                "task_count",
            ],
        )
        .context("open jobs appender")?;
    let mut tasks = conn
        .appender_with_columns(
            "tasks",
            &[
                "task_id",
                "job_id",
                "started_at",
                "finished_at",
                "target_uri",
                "command",
                "status",
                "exit_code",
                "error",
                "connect_ms",
                "auth_ms",
                "exec_ms",
            ],
        )
        .context("open tasks appender")?;
    let mut task_vars = conn
        .appender_with_columns("task_vars", &["task_id", "key", "value"])
        .context("open task_vars appender")?;
    let mut task_lines = conn
        .appender_with_columns("task_lines", &["task_id", "stream", "seq", "line_hash"])
        .context("open task_lines appender")?;

    let mut pending_jobs: HashMap<Vec<u8>, PendingJob> = HashMap::new();
    let mut pending_tasks: HashMap<Vec<u8>, PendingTask> = HashMap::new();
    let mut line_dict_batch: HashMap<[u8; 16], String> = HashMap::new();

    while let Some(event) = rx.blocking_recv() {
        match event {
            WriteEvent::JobStart {
                job_id,
                started_at,
                command,
                concurrency,
                task_count,
            } => {
                pending_jobs.insert(
                    job_id,
                    PendingJob {
                        started_at,
                        command,
                        concurrency,
                        task_count,
                    },
                );
            }
            WriteEvent::JobEnd {
                job_id,
                finished_at,
            } => {
                let Some(job) = pending_jobs.remove(&job_id) else {
                    return Err(anyhow!("missing JobStart for JobEnd"));
                };
                jobs.append_row(params![
                    job_id.clone(),
                    job.started_at,
                    finished_at,
                    job.command,
                    job.concurrency,
                    job.task_count
                ])
                .context("append job row")?;
                conn.execute(
                    "INSERT INTO meta(key, value) VALUES ('last_job_id', ?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    params![job_id],
                )
                .context("update last_job_id")?;
            }
            WriteEvent::TaskStart {
                task_id,
                job_id,
                started_at,
                target_uri,
                command,
            } => {
                pending_tasks.insert(
                    task_id,
                    PendingTask {
                        job_id,
                        started_at,
                        target_uri,
                        command,
                    },
                );
            }
            WriteEvent::TaskEnd {
                task_id,
                finished_at,
                status,
                exit_code,
                error,
                connect_ms,
                auth_ms,
                exec_ms,
            } => {
                let Some(task) = pending_tasks.remove(&task_id) else {
                    return Err(anyhow!("missing TaskStart for TaskEnd"));
                };
                tasks
                    .append_row(params![
                        task_id,
                        task.job_id,
                        task.started_at,
                        finished_at,
                        task.target_uri,
                        task.command,
                        status,
                        exit_code,
                        error,
                        connect_ms,
                        auth_ms,
                        exec_ms,
                    ])
                    .context("append task row")?;
            }
            WriteEvent::TaskVar {
                task_id,
                key,
                value,
            } => {
                task_vars
                    .append_row(params![task_id, key, value])
                    .context("append task var")?;
            }
            WriteEvent::TaskLines {
                task_id,
                stream,
                lines,
            } => {
                for (seq, line_hash, line_text) in lines {
                    task_lines
                        .append_row(params![
                            task_id.clone(),
                            stream.as_str(),
                            seq,
                            line_hash.to_vec()
                        ])
                        .context("append task line")?;
                    line_dict_batch.entry(line_hash).or_insert(line_text);
                }
                if line_dict_batch.len() >= 100_000 {
                    flush_line_dict_batch(&conn, &mut line_dict_batch)?;
                }
            }
            WriteEvent::Sync { ack } => {
                let result = (|| -> Result<()> {
                    jobs.flush().context("flush jobs")?;
                    tasks.flush().context("flush tasks")?;
                    task_vars.flush().context("flush task_vars")?;
                    task_lines.flush().context("flush task_lines")?;
                    flush_line_dict_batch(&conn, &mut line_dict_batch)?;
                    Ok(())
                })();
                let _ = ack.send(result);
            }
        }
    }

    jobs.flush().context("flush jobs at shutdown")?;
    tasks.flush().context("flush tasks at shutdown")?;
    task_vars.flush().context("flush task_vars at shutdown")?;
    task_lines.flush().context("flush task_lines at shutdown")?;
    flush_line_dict_batch(&conn, &mut line_dict_batch)?;
    Ok(())
}

fn flush_line_dict_batch(conn: &Connection, batch: &mut HashMap<[u8; 16], String>) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn.unchecked_transaction().context("begin line_dict tx")?;
    let mut stmt = tx
        .prepare("INSERT INTO line_dict(line_hash, line_text) VALUES (?, ?) ON CONFLICT(line_hash) DO NOTHING")
        .context("prepare line_dict insert")?;
    for (line_hash, line_text) in batch.drain() {
        stmt.execute(params![line_hash.to_vec(), line_text])
            .context("insert line_dict")?;
    }
    drop(stmt);
    tx.commit().context("commit line_dict tx")?;
    Ok(())
}

fn init_schema(conn: &Connection) -> Result<()> {
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
    )
    .context("init schema")?;
    Ok(())
}

#[async_trait]
impl Db for DuckDb {
    async fn migrate(&self) -> Result<()> {
        self.sync().await
    }

    async fn save(&self, entry: &ResultEntry) -> Result<()> {
        // Compatibility path: map legacy result rows into tasks table.
        let task_id = Uuid::now_v7().hyphenated().to_string();
        self.create_task(&task_id, &entry.job_id, &entry.target, "")
            .await?;

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
            let exit_status = exit_map
                .get(&row.task_id)
                .and_then(|x| x.parse::<u32>().ok());
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
