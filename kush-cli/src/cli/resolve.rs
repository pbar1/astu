use clap::Args;
use futures::StreamExt;

use crate::argetype::ResolutionArgs;

/// Resolve targets from queries
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,
}

#[async_trait::async_trait]
impl super::Run for ResolveArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let mut targets = self.resolution_args.clone().resolve();

        while let Some(target) = targets.next().await {
            println!("{target}");
        }

        Ok(())
    }
}
