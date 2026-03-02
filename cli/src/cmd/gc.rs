use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Garbage collect old persisted data.
#[derive(Debug, Args)]
pub struct GcArgs {}

impl Run for GcArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        todo!()
    }
}
