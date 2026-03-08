use clap::Args;

use crate::arg::ActionFlags;

#[derive(Debug, Args)]
pub struct Ping {
    #[command(flatten)]
    pub action: ActionFlags,
}
