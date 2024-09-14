use anyhow::Result;
use astu_util::id::Id;
use clap::Args;
use futures::StreamExt;

use crate::argetype::ResolutionArgs;
use crate::cmd::Run;

/// Resolve targets
#[derive(Debug, Args)]
pub struct Resolve {
    #[command(flatten)]
    resolution_args: ResolutionArgs,
}

impl Run for Resolve {
    async fn run(&self, _id: Id) -> Result<()> {
        let mut targets = self.resolution_args.clone().resolve();

        while let Some(target) = targets.next().await {
            println!("{target}");
        }

        Ok(())
    }
}
