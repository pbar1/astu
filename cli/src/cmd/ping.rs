use anyhow::Result;
use astu_util::id::Id;
use clap::Args;

use crate::argetype::ConnectionArgs;
use crate::argetype::ResolutionArgs;
use crate::cmd::Run;

/// Connect to targets
#[derive(Debug, Args)]
pub struct Ping {
    #[command(flatten)]
    resolution_args: ResolutionArgs,

    #[command(flatten)]
    connection_args: ConnectionArgs,
}

impl Run for Ping {
    async fn run(&self, _id: Id) -> Result<()> {
        let _targets = self.resolution_args.clone().resolve();

        Ok(())
    }
}
