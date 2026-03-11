use clap::Args;

/// Clean old jobs and associated data from the database
///
/// Cleans the database of jobs and their associated data.
#[derive(Debug, Args)]
pub struct Gc {
    /// Delete data that was collected at this age or older.
    #[arg(long, value_name = "DURATION")]
    pub before: Option<String>,
}

impl crate::Run for Gc {
    async fn run(&self) -> eyre::Result<()> {
        eyre::bail!("unimplemented")
    }
}
