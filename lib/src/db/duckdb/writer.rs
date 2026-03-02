use std::collections::HashMap;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use duckdb::params;
use duckdb::Connection;
use tokio::sync::mpsc;

use super::schema;
use super::WriteEvent;

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

pub(super) fn run_writer_loop(path: &Path, rx: &mut mpsc::Receiver<WriteEvent>) -> Result<()> {
    let conn = Connection::open(path).context("open duckdb")?;
    schema::init_schema(&conn)?;

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
