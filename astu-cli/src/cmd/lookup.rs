use clap::Args;

/// Resolve targets
///
/// Expands a set of input targets into a set of actionable targets. Does not
/// display a plan, and no actual actions are performed on targets.
#[derive(Debug, Args)]
pub struct Lookup {
    #[command(flatten)]
    pub action: crate::arg::ActionFlags,
}
