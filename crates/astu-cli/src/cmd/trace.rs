use clap::Args;

use crate::arg::ResultFlags;

/// Display diagnostic timing traces for tasks in a job
///
/// Displays a diagnostic trace of timings for the sequence of actions and
/// observed errors for tasks in a job.
#[derive(Debug, Args)]
pub struct Trace {
    #[command(flatten)]
    pub result: ResultFlags,

    /// Target URI filter.
    #[arg(short = 'T', long, value_name = "TARGET")]
    pub target: Vec<String>,
}

impl crate::Run for Trace {
    async fn run(&self) -> eyre::Result<()> {
        eyre::bail!("unimplemented")
    }
}
