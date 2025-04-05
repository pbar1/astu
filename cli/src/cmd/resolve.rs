use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;

use crate::argetype::ResolutionArgs;
use crate::cmd::Run;

/// Resolve targets
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[clap(flatten)]
    resolution_args: ResolutionArgs,

    /// Print the target graph as GraphViz.
    #[clap(short = 'g', long)]
    graph: bool,
}

impl Run for ResolveArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        if self.graph {
            let targets = self.resolution_args.clone().graph_full().await?;
            let dot = targets.graphviz();
            println!("{dot}");
        } else {
            let targets = self.resolution_args.clone().set().await?;
            for target in targets {
                println!("{target}");
            }
        }
        Ok(())
    }
}
