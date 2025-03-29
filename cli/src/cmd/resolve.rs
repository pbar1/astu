use anyhow::Result;
use astu_util::id::Id;
use clap::Args;

use crate::argetype::ResolutionArgs;
use crate::cmd::Run;

/// Resolve targets
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[clap(flatten)]
    resolution_args: ResolutionArgs,
}

impl Run for ResolveArgs {
    async fn run(&self, _id: Id) -> Result<()> {
        let targets = self.resolution_args.clone().resolve().await;

        let dot = targets.graphviz();
        println!("{dot}");

        Ok(())
    }
}
