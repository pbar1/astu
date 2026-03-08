use clap::Args;

use crate::arg::ResultField;
use crate::arg::ResultFlags;

/// Display per-task captured output for a job
///
/// Displays tables of captured output per task in a job.
#[derive(Debug, Args)]
pub struct Output {
    #[command(flatten)]
    pub result: ResultFlags,

    /// Target URI filter.
    #[arg(short = 'T', long, value_name = "TARGET")]
    pub target: Vec<String>,

    /// Filter rows to values containing this string.
    #[arg(long)]
    pub value: Option<String>,

    /// Restrict output to these fields.
    #[arg(
        value_name = "FIELD",
        default_values = ["status", "stdout", "stderr", "exitcode"]
    )]
    pub field: Vec<ResultField>,
}
