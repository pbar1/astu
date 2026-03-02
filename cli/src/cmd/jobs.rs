use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Display jobs metadata.
#[derive(Debug, Args)]
pub struct JobsArgs {}

impl Run for JobsArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        todo!()
    }
}
