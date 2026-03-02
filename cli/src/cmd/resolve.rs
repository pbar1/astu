use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;

/// Resolve targets.
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[clap(flatten)]
    resolution_args: crate::args::ResolutionArgs,
}

impl Run for ResolveArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        for target in self.resolution_args.set_with_default(None).await? {
            println!("{target}");
        }

        Ok(())
    }
}
