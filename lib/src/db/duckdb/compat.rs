use anyhow::Result;
use duckdb::params;

use super::uuid_string_to_blob;
use super::DuckDb;
use crate::db::ResultEntry;

pub(super) async fn save_compat(db: &DuckDb, entry: &ResultEntry) -> Result<()> {
    let task_id = uuid::Uuid::now_v7().hyphenated().to_string();
    db.create_task(&task_id, &entry.job_id, &entry.target, "")
        .await?;

    if let Some(stdout) = &entry.stdout {
        db.append_stream_blob(&task_id, "stdout", stdout).await?;
    }
    if let Some(stderr) = &entry.stderr {
        db.append_stream_blob(&task_id, "stderr", stderr).await?;
    }

    let status = if entry.error.is_some() || entry.exit_status.unwrap_or(0) != 0 {
        super::DbTaskStatus::Failed
    } else {
        super::DbTaskStatus::Complete
    };

    db.finish_task(
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

pub(super) async fn load_compat(db: &DuckDb, job_id: &str) -> Result<Vec<ResultEntry>> {
    let job_blob = uuid_string_to_blob(job_id)?;
    db.sync().await?;
    let conn = db.read_conn()?;
    let mut stmt = conn.prepare(
        "WITH stdout_assembled AS (
            SELECT tl.task_id, string_agg(ld.line_text, '\n' ORDER BY tl.seq) AS stdout_value
            FROM task_lines tl
            JOIN line_dict ld ON ld.line_hash = tl.line_hash
            WHERE tl.stream='stdout'
            GROUP BY tl.task_id
         ),
         stderr_assembled AS (
            SELECT tl.task_id, string_agg(ld.line_text, '\n' ORDER BY tl.seq) AS stderr_value
            FROM task_lines tl
            JOIN line_dict ld ON ld.line_hash = tl.line_hash
            WHERE tl.stream='stderr'
            GROUP BY tl.task_id
         )
         SELECT t.target_uri, t.error, t.exit_code, s.stdout_value, e.stderr_value
         FROM tasks t
         LEFT JOIN stdout_assembled s ON s.task_id=t.task_id
         LEFT JOIN stderr_assembled e ON e.task_id=t.task_id
         WHERE t.job_id=?
         ORDER BY t.started_at ASC",
    )?;
    let mut rows = stmt.query(params![job_blob])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(ResultEntry {
            job_id: job_id.to_owned(),
            target: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
            error: row.get::<_, Option<String>>(1)?,
            exit_status: row
                .get::<_, Option<i64>>(2)?
                .and_then(|x| u32::try_from(x).ok()),
            stdout: row.get::<_, Option<String>>(3)?.map(String::into_bytes),
            stderr: row.get::<_, Option<String>>(4)?.map(String::into_bytes),
        });
    }
    Ok(out)
}
