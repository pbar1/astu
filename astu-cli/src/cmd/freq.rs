use clap::Args;

use crate::{
    arg::ResultFlags,
    cmd::result_field::ResultField,
};

#[derive(Debug, Args)]
pub struct Freq {
    #[command(flatten)]
    pub result: ResultFlags,

    #[arg(long)]
    pub contains: Option<String>,

    #[arg(value_name = "FIELD")]
    pub field: Option<ResultField>,
}
