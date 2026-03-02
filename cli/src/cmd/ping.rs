use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use uuid::Uuid;

use crate::cmd::Run;

/// Connect to targets
#[derive(Debug, Args)]
pub struct PingArgs {
    #[clap(flatten)]
    resolution_args: crate::args::ResolutionArgs,

    #[clap(flatten)]
    action_args: crate::args::ActionArgs,
}

impl Run for PingArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let targets = self.resolution_args.set_with_default(None).await?;
        self.action_args.require_confirm(targets.len())?;

        let job_id = Uuid::now_v7().hyphenated().to_string();
        let specs = targets
            .into_iter()
            .map(|target| crate::args::TaskSpec {
                target,
                command: "ping".to_owned(),
                param: None,
            })
            .collect::<Vec<_>>();
        self.action_args
            .run_tasks_for_operation(
                db.clone(),
                &job_id,
                specs,
                crate::args::ActionOperation::Ping,
            )
            .await?;

        let DbImpl::Duck(db) = db;
        crate::report::print_error_freq_summary(&db, &job_id).await?;
        Ok(())
    }
}
