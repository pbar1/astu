use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Replay outputs from a prior run.
#[derive(Debug, Args)]
pub struct OutputArgs {}

impl Run for OutputArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        todo!()
    }
}
