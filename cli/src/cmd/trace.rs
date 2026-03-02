use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Display task trace timings and errors.
#[derive(Debug, Args)]
pub struct TraceArgs {
    #[arg(short = 'j', long)]
    job: Option<String>,

    #[arg(short = 'T', long = "target")]
    target: Option<String>,
}

impl Run for TraceArgs {
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

        let rows = db.trace(&job_id, self.target.as_deref()).await?;
        println!("task_id\ttarget\tstatus\terror\tconnect_ms\tauth_ms\texec_ms");
        for row in rows {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                row.task_id,
                row.target,
                row.status,
                row.error,
                row.connect_ms,
                row.auth_ms,
                row.exec_ms,
            );
        }
        Ok(())
    }
}
