use clap::Args;
use url::Url;

#[derive(Debug, Args)]
pub(crate) struct CpArgs {
    /// Source location
    source: Url,

    /// Target location
    target: Url,
}
