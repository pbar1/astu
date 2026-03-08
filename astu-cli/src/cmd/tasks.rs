use clap::Args;

use crate::arg::ResultFlags;

#[derive(Debug, Args)]
pub struct Tasks {
    #[command(flatten)]
    pub result: ResultFlags,
}
