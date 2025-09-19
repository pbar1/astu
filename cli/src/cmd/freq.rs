use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Aggregate results from a prior run into frequency tables.
#[derive(Debug, Args)]
pub struct FreqArgs {}

impl Run for FreqArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        todo!()
    }
}
