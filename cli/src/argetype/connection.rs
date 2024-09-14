use clap::Args;

const HEADING: Option<&str> = Some("Connection Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ConnectionArgs {
    /// Time in seconds to allow connection to complete.
    #[arg(long, default_value_t = 2, help_heading = HEADING)]
    pub connect_timeout: u64,

    /// Bind all connections to the same local address.
    ///
    /// This greatly increases the possible number of concurrent connections, at
    /// the cost of being unable to create more than one simultaneous connection
    /// to each remote address.
    #[arg(long, help_heading = HEADING)]
    pub reuseport: bool,

    // TODO: Remove if we get IP connection (ie, ICMP)
    /// Port to connect to if not provided by target
    #[arg(long, default_value_t = 22, help_heading = HEADING)]
    pub port: u16,
}
