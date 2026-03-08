use clap::Args;

use crate::arg::ResultField;
use crate::arg::ResultFlags;

/// Display aggregated result frequencies for a job
///
/// Displays tables of captured output aggregated by count of appearance in a
/// job.
#[derive(Debug, Args)]
pub struct Freq {
    #[command(flatten)]
    pub result: ResultFlags,

    /// Filter rows to values containing this string.
    #[arg(long)]
    pub contains: Option<String>,

    /// Restrict output to these fields.
    #[arg(
        value_name = "FIELD",
        default_values = ["status", "stdout", "stderr", "exitcode"]
    )]
    pub field: Vec<ResultField>,
}
