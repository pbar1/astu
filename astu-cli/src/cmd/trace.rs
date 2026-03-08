use clap::Args;

use crate::arg::ResultFlags;

#[derive(Debug, Args)]
pub struct Trace {
    #[command(flatten)]
    pub result: ResultFlags,

    #[arg(short = 'T', long, value_name = "TARGET")]
    pub target: Vec<String>,
}
