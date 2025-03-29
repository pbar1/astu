use std::sync::Arc;

use anyhow::Result;
use astu_util::tcp_stream::DefaultTcpStreamFactory;
use astu_util::tcp_stream::ReuseportTcpStreamFactory;
use astu_util::tcp_stream::TcpStreamFactory;
use clap::Args;

const HEADING: Option<&str> = Some("Connection Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ConnectionArgs {
    /// Time in seconds to allow connection to complete.
    #[clap(long, default_value_t = 2, help_heading = HEADING)]
    pub connect_timeout: u64,

    /// Bind all connections to the same local address.
    ///
    /// This greatly increases the possible number of concurrent connections, at
    /// the cost of being unable to create more than one simultaneous connection
    /// to each remote address.
    #[clap(long, help_heading = HEADING)]
    pub reuseport: bool,

    // TODO: Remove if we get IP connection (ie, ICMP)
    /// Port to connect to if not provided by target.
    #[clap(long, default_value_t = 22, help_heading = HEADING)]
    pub port: u16,

    /// Number of concurrent connections to process.
    #[clap(short = 'c', long, default_value_t = 50000, help_heading = HEADING)]
    pub concurrency: usize,
}

impl ConnectionArgs {
    pub fn tcp_stream_factory(&self) -> Result<Arc<dyn TcpStreamFactory + Send + Sync>> {
        let factory: Arc<dyn TcpStreamFactory + Send + Sync> = if self.reuseport {
            Arc::new(ReuseportTcpStreamFactory::try_new()?)
        } else {
            Arc::new(DefaultTcpStreamFactory)
        };
        Ok(factory)
    }
}
