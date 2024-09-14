use astu_util::id::Id;
use clap::Args;
use futures::StreamExt;

use crate::argetype::ResolutionArgs;

/// Resolve targets from queries
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,
}

impl super::Run for ResolveArgs {
    async fn run(&self, id: Id) -> anyhow::Result<()> {
        eprintln!("Invocation ID: {id}");

        let mut targets = self.resolution_args.clone().resolve();

        while let Some(target) = targets.next().await {
            println!("{target}");
        }

        Ok(())
    }
}
