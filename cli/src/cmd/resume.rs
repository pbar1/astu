use std::str::FromStr;

use anyhow::Result;
use astu::db::DbImpl;
use astu::resolve::Target;
use astu::util::id::Id;
use clap::Args;
use uuid::Uuid;

use crate::cmd::common;
use crate::cmd::Run;

/// Resume a canceled job.
#[derive(Debug, Args)]
pub struct ResumeArgs {
    #[arg(short = 'j', long)]
    job: Option<String>,

    #[command(flatten)]
    auth_args: crate::args::AuthArgs,

    #[command(flatten)]
    action_args: crate::args::ActionArgs,
}

impl Run for ResumeArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let DbImpl::Duck(duck) = db.clone();
        let job_id = if let Some(job) = &self.job {
            job.clone()
        } else {
            let Some(job) = duck.last_job_id().await? else {
                return Ok(());
            };
            job
        };

        let canceled = duck.canceled_tasks_for_job(&job_id).await?;
        if canceled.is_empty() {
            println!("No canceled tasks to resume for job: {job_id}");
            return Ok(());
        }

        let command = duck.command_for_job(&job_id).await?.unwrap_or_default();
        println!("Resuming job: {job_id}");
        println!("Command: {command}");

        let specs = canceled
            .into_iter()
            .map(|(_task_id, target, task_command)| {
                let target = Target::from_str(&target)?;
                let effective_command = if task_command.is_empty() {
                    command.clone()
                } else {
                    task_command
                };
                Ok(common::TaskSpec {
                    target,
                    command: effective_command,
                    param: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let new_job = Uuid::now_v7().hyphenated().to_string();
        common::run_tasks(
            db,
            &new_job,
            specs,
            &self.auth_args,
            &self.action_args,
            None,
        )
        .await
    }
}
