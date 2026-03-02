use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use tabled::Tabled;

use crate::cmd::Run;

/// Display jobs metadata.
#[derive(Debug, Args)]
pub struct JobsArgs {
    #[arg(long, default_value_t = 50)]
    limit: i64,
}

#[derive(Debug, Tabled)]
struct JobRowView {
    job_id: String,
    started_at: String,
    task_count: i64,
    command: String,
}

impl Run for JobsArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let DbImpl::Duck(db) = db;
        let rows = db.jobs(self.limit).await?;
        let view = rows
            .into_iter()
            .map(|row| JobRowView {
                job_id: row.job_id,
                started_at: row.started_at,
                task_count: row.task_count,
                command: row.command,
            })
            .collect::<Vec<_>>();
        let rendered = crate::cmd::render::markdown_table(view);
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;
        Ok(())
    }
}
