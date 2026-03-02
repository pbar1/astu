use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Display tasks metadata for a job.
#[derive(Debug, Args)]
pub struct TasksArgs {}

impl Run for TasksArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        todo!()
    }
}
