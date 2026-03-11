use clap::Args;

/// Display jobs and job metadata
///
/// Displays a table of jobs and their metadata.
#[derive(Debug, Args)]
pub struct Jobs {}

impl crate::Run for Jobs {
    async fn run(&self) -> eyre::Result<()> {
        eyre::bail!("unimplemented")
    }
}
