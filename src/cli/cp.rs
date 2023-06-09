use clap::Args;
use url::Url;

#[derive(Debug, Args)]
pub struct CpArgs {
    /// Source location
    source: Url,

    /// Target location
    target: Url,
}
