use anyhow::Result;
use chrono::Duration;
use chrono::Utc;
use duckdb::params;

use super::DuckDb;

pub(super) async fn gc_before(db: &DuckDb, before: Duration) -> Result<()> {
    db.sync().await?;
    let cutoff = (Utc::now() - before).to_rfc3339();
    let conn = db.read_conn()?;

    let mut old_jobs = Vec::<Vec<u8>>::new();
    let mut stmt = conn.prepare("SELECT job_id FROM jobs WHERE started_at <= ?")?;
    let mut rows = stmt.query(params![cutoff])?;
    while let Some(row) = rows.next()? {
        old_jobs.push(row.get(0)?);
    }

    for job_blob in old_jobs {
        conn.execute(
            "DELETE FROM task_lines WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id=?)",
            params![job_blob.clone()],
        )?;
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
