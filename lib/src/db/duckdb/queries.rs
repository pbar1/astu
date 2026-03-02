use anyhow::Result;
use duckdb::params;

use super::uuid_string_to_blob;
use super::DbField;
use super::DuckDb;
use super::FreqRow;
use super::OutputRow;

pub(super) async fn freq(
    db: &DuckDb,
    field: DbField,
    job_id: &str,
    contains: Option<&str>,
) -> Result<Vec<FreqRow>> {
    let limit = 200_i64;
    let job_blob = uuid_string_to_blob(job_id)?;
    db.sync().await?;
    let conn = db.read_conn()?;
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

pub(super) async fn output(
    db: &DuckDb,
    field: DbField,
    job_id: &str,
    contains: Option<&str>,
    target_filter: Option<&str>,
) -> Result<Vec<OutputRow>> {
    let job_blob = uuid_string_to_blob(job_id)?;
    let contains = contains.unwrap_or("");
    let target = target_filter.unwrap_or("");
    db.sync().await?;
    let conn = db.read_conn()?;

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
            task_id: super::uuid_blob_to_string(&row.get::<_, Vec<u8>>(0)?)?,
            target: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            value: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        });
    }
    Ok(out)
}
