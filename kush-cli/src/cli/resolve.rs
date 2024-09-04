use clap::Args;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use kush_resolve::Target;

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
        let targets = self.resolution_args.clone().resolve();
        process_targets(targets).await;
        Ok(())
    }
}

async fn process_targets(targets: impl Stream<Item = Target>) {
    pin_mut!(targets);
    while let Some(target) = targets.next().await {
        println!("{target}");
    }
}
