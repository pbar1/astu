use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Resume a canceled job.
#[derive(Debug, Args)]
pub struct ResumeArgs {}

impl Run for ResumeArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        todo!()
    }
}
