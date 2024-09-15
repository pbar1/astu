use anyhow::Result;
use astu_util::id::Id;
use clap::Args;
use futures::StreamExt;

use crate::argetype::ResolutionArgs;
use crate::cmd::Run;

/// Resolve targets
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,
}

impl Run for ResolveArgs {
    async fn run(&self, _id: Id) -> Result<()> {
        let mut targets = self.resolution_args.clone().resolve();

        // TODO: Implement with Store
        while let Some(target) = targets.next().await {
            println!("{target}");
        }

        Ok(())
    }
}
