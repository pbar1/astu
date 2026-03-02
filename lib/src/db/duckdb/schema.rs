use anyhow::Context;
use anyhow::Result;
use duckdb::Connection;

pub(super) fn init_schema(conn: &Connection) -> Result<()> {
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
CREATE INDEX IF NOT EXISTS idx_tasks_job_started ON tasks(job_id, started_at);
CREATE INDEX IF NOT EXISTS idx_tasks_job_target ON tasks(job_id, target_uri);
CREATE INDEX IF NOT EXISTS idx_task_lines_task_stream_seq ON task_lines(task_id, stream, seq);
CREATE INDEX IF NOT EXISTS idx_task_lines_stream_task ON task_lines(stream, task_id);
CREATE INDEX IF NOT EXISTS idx_task_vars_task ON task_vars(task_id);
",
    )
    .context("init schema")?;
    Ok(())
}
