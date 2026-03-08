use clap::Args;

use crate::arg::ResultFlags;
use crate::cmd::result_field::ResultField;

#[derive(Debug, Args)]
pub struct Output {
    #[command(flatten)]
    pub result: ResultFlags,

    #[arg(short = 'T', long, value_name = "TARGET")]
    pub target: Vec<String>,

    #[arg(long)]
    pub contains: Option<String>,

    #[arg(value_name = "FIELD")]
    pub field: Option<ResultField>,
}
