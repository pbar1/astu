use clap::Args;

use crate::arg::ResultFlags;

/// Display tasks and task metadata for a job
///
/// Displays a table of tasks and their metadata within a job.
#[derive(Debug, Args)]
pub struct Tasks {
    #[command(flatten)]
    pub result: ResultFlags,
}

impl crate::Run for Tasks {
    async fn run(&self) -> eyre::Result<()> {
        eyre::bail!("unimplemented")
    }
}
