use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Display task trace timings and errors.
#[derive(Debug, Args)]
pub struct TraceArgs {}

impl Run for TraceArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        todo!()
    }
}
