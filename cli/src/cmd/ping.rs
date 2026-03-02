use std::str::FromStr;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use astu::action::Client;
use astu::action::ClientFactory;
use astu::db::DbImpl;
use astu::db::DbTaskStatus;
use astu::resolve::Target;
use astu::util::id::Id;
use clap::Args;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::cmd::common;
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
        let mut set = self.resolution_args.set().await?;
        if set.is_empty() {
            set.insert(Target::from_str("local:")?);
        }
        common::require_confirm(self.action_args.confirm, set.len())?;

        let job_id = Uuid::now_v7().hyphenated().to_string();
        let db = match db {
            DbImpl::Duck(db) => db,
        };
        db.create_job(
            &job_id,
            "astu ping",
            i64::try_from(self.action_args.concurrency).unwrap_or(i64::MAX),
            i64::try_from(set.len()).unwrap_or(i64::MAX),
        )
        .await?;

        let sem = Arc::new(Semaphore::new(self.action_args.concurrency.max(1)));
        let factory = self.action_args.client_factory()?;
        let mut joins = tokio::task::JoinSet::new();

        for target in set {
            let permit = sem.clone().acquire_owned().await?;
            let target_s = target.to_string();
            let db = db.clone();
            let factory = factory.clone();
            let job_id_for_task = job_id.clone();
            joins.spawn(async move {
                let _permit = permit;
                let task_id = Uuid::now_v7().hyphenated().to_string();
                db.create_task(&task_id, &job_id_for_task, &target_s, "ping")
                    .await?;

                let mut client = factory.client(&target).context("failed getting client")?;

                let t_connect = Instant::now();
                if let Err(error) = client.connect().await {
                    db.finish_task(
                        &task_id,
                        DbTaskStatus::Failed,
                        None,
                        Some(&format!("{error:#}")),
                        i64::try_from(t_connect.elapsed().as_millis()).unwrap_or(i64::MAX),
                        0,
                        0,
                    )
                    .await?;
                    return Ok::<(), anyhow::Error>(());
                }
                let connect_ms = i64::try_from(t_connect.elapsed().as_millis()).unwrap_or(i64::MAX);

                let t_ping = Instant::now();
                match client.ping().await {
                    Ok(stdout) => {
                        db.append_stream_blob(&task_id, "stdout", &stdout).await?;
                        db.finish_task(
                            &task_id,
                            DbTaskStatus::Complete,
                            Some(0),
                            None,
                            connect_ms,
                            0,
                            i64::try_from(t_ping.elapsed().as_millis()).unwrap_or(i64::MAX),
                        )
                        .await?;
                    }
                    Err(error) => {
                        db.finish_task(
                            &task_id,
                            DbTaskStatus::Failed,
                            None,
                            Some(&format!("{error:#}")),
                            connect_ms,
                            0,
                            i64::try_from(t_ping.elapsed().as_millis()).unwrap_or(i64::MAX),
                        )
                        .await?;
                    }
                }

                Ok::<(), anyhow::Error>(())
            });
        }

        while let Some(done) = joins.join_next().await {
            done??;
        }

        db.finish_job(&job_id).await?;
        Ok(())
    }
}
