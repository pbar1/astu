use anyhow::Result;
use astu_util::id::Id;
use clap::Args;

use crate::argetype::ConnectionArgs;
use crate::argetype::ResolutionArgs;
use crate::cmd::Run;

/// Run commands on targets
#[derive(Debug, Args)]
pub struct Exec {
    #[command(flatten)]
    resolution_args: ResolutionArgs,

    #[command(flatten)]
    connection_args: ConnectionArgs,

    /// Command to run.
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,

    /// Remote user to authenticate as.
    #[arg(short = 'u', long, default_value = "root")]
    user: String,

    /// SSH agent socket to use.
    #[arg(long, env = "SSH_AUTH_SOCK")]
    ssh_agent: Option<String>,
}

impl Run for Exec {
    async fn run(&self, _id: Id) -> Result<()> {
        let _targets = self.resolution_args.clone().resolve();

        Ok(())
    }
}
