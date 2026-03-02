use anyhow::Result;
use astu::db::DuckDb;
use clap::Args;

#[derive(Debug, Args, Clone)]
pub struct JobArgs {
    #[arg(short = 'j', long)]
    pub job: Option<String>,
}

impl JobArgs {
    pub async fn resolve(&self, db: &DuckDb) -> Result<Option<String>> {
        if let Some(job) = &self.job {
            Ok(Some(job.clone()))
        } else {
            db.last_job_id().await
        }
    }
}
