use clap::Args;

use crate::arg::ActionFlags;

/// Ping targets
///
/// Performs this sequence of actions on each target in the set:
/// - Connect
/// - Ping
///
/// Persists the output of ping (if it exists) as stdout, as well as the timing
/// of each phase. Exitcode and stderr will never exist.
#[derive(Debug, Args)]
pub struct Ping {
    #[command(flatten)]
    pub action: ActionFlags,
}

impl crate::Run for Ping {
    async fn run(&self) -> eyre::Result<()> {
        eyre::bail!("unimplemented")
    }
}
