use clap::Args;

/// Resolve targets
#[derive(Debug, Args)]
pub struct Lookup {
    #[command(flatten)]
    pub action: crate::arg::ActionFlags,
}
