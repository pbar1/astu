use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Display tasks metadata for a job.
#[derive(Debug, Args)]
pub struct TasksArgs {
    #[arg(short = 'j', long)]
    job: Option<String>,
}

impl Run for TasksArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let DbImpl::Duck(db) = db;
        let job_id = if let Some(job) = &self.job {
            job.clone()
        } else {
            let Some(job) = db.last_job_id().await? else {
                return Ok(());
            };
            job
        };

        let rows = db.tasks(&job_id).await?;
        println!("task_id\ttarget\tstatus\tcommand\texit_code");
        for row in rows {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                row.task_id,
                row.target,
                row.status,
                row.command,
                row.exit_code.map_or_else(String::new, |x| x.to_string())
            );
        }
        Ok(())
    }
}
