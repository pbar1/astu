use std::sync::Arc;

use anyhow::Result;
use clap::Args;

const HEADING: Option<&str> = Some("Connection Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ConnectionArgs {
    /// Time in seconds to allow connection to complete.
    #[clap(long, default_value_t = 30, help_heading = HEADING)]
    pub connect_timeout: u64,

    /// Number of concurrent connections to process.
    #[clap(short = 'c', long, default_value_t = 500, help_heading = HEADING)]
    pub concurrency: usize,
}

impl ConnectionArgs {}
