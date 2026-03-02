use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use chrono::Duration;
use clap::Args;

use crate::cmd::Run;

/// Garbage collect old persisted data.
#[derive(Debug, Args)]
pub struct GcArgs {
    #[arg(long)]
    before: humantime::Duration,
}

impl Run for GcArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let DbImpl::Duck(db) = db;
        let secs = i64::try_from(self.before.as_secs()).unwrap_or(i64::MAX);
        db.gc_before(Duration::seconds(secs)).await
    }
}
