use clap::Args;
use url::Url;

use super::Runnable;

#[derive(Debug, Args)]
pub struct CpArgs {
    /// Source location
    source: Url,

    /// Target location
    target: Url,
}

impl Runnable for CpArgs {
    fn run(&self) -> anyhow::Result<()> {
        dbg!(&self);
        todo!()
    }
}
