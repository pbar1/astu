use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use tabled::Tabled;

use crate::cmd::Run;

/// Display tasks metadata for a job.
#[derive(Debug, Args)]
pub struct TasksArgs {
    #[command(flatten)]
    job_args: crate::args::JobArgs,
}

#[derive(Debug, Tabled)]
struct TaskRowView {
    task_id: String,
    target: String,
    status: String,
    command: String,
    exit_code: String,
}

impl Run for TasksArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let DbImpl::Duck(db) = db;
        let Some(job_id) = self.job_args.resolve(&db).await? else {
            return Ok(());
        };

        let rows = db.tasks(&job_id).await?;
        let view = rows
            .into_iter()
            .map(|row| TaskRowView {
                task_id: row.task_id,
                target: row.target,
                status: row.status,
                command: row.command,
                exit_code: row.exit_code.map_or_else(String::new, |x| x.to_string()),
            })
            .collect::<Vec<_>>();
        let rendered = crate::cmd::render::markdown_table(view);
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;
        Ok(())
    }
}
