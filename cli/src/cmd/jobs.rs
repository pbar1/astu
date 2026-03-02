use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Display jobs metadata.
#[derive(Debug, Args)]
pub struct JobsArgs {
    #[arg(long, default_value_t = 50)]
    limit: i64,
}

impl Run for JobsArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let DbImpl::Duck(db) = db;
        let rows = db.jobs(self.limit).await?;
        println!("job_id\tstarted_at\tfinished_at\ttask_count\tcommand");
        for row in rows {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                row.job_id, row.started_at, row.finished_at, row.task_count, row.command
            );
        }
        Ok(())
    }
}
