use anyhow::Result;
use astu::db::DbImpl;
use astu::resolve::Target;
use astu::util::id::Id;
use clap::Args;
use std::str::FromStr;

use crate::cmd::Run;

/// Resolve targets.
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[clap(flatten)]
    resolution_args: crate::args::ResolutionArgs,
}

impl Run for ResolveArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        let mut set = self.resolution_args.set().await?;
        if set.is_empty() {
            set.insert(Target::from_str("local:")?);
        }

        for target in set {
            println!("{target}");
        }

        Ok(())
    }
}
