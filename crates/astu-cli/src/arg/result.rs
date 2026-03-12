use clap::Args;
use clap::ValueEnum;

#[derive(Debug, Clone, Default, Args)]
pub struct ResultFlags {
    /// Job ID to display results for.
    ///
    /// If not set, uses the last action job ID persisted in the DB.
    #[arg(
        short = 'j',
        long,
        value_name = "JOB_ID",
        help_heading = "Result Flags"
    )]
    pub job: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ResultField {
    /// Task outcome
    Status,

    /// Task primary output
    Stdout,

    /// Task secondary output
    Stderr,

    /// Task exit code
    Exitcode,
}
