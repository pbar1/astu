use anyhow::Result;
use astu::util::id::Id;
use chrono::Duration;
use clap::Args;

use crate::cmd::Run;
use crate::runtime::Runtime;

/// Garbage collect old persisted data.
#[derive(Debug, Args)]
pub struct GcArgs {
    #[arg(long)]
    before: humantime::Duration,
}

impl Run for GcArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let secs = i64::try_from(self.before.as_secs()).unwrap_or(i64::MAX);
        runtime.db().gc_before(Duration::seconds(secs)).await
    }
}
