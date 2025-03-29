use std::sync::Arc;

use anyhow::Result;
use astu_action::client;
use astu_action::transport;
use clap::Args;
use humantime::Duration;

const HEADING: Option<&str> = Some("Connection Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ConnectionArgs {
    /// Time in seconds to allow connection to complete.
    #[clap(long, default_value = "30s", help_heading = HEADING)]
    pub connect_timeout: Duration,

    /// Number of concurrent connections to process.
    #[clap(short = 'c', long, default_value_t = 500, help_heading = HEADING)]
    pub concurrency: usize,
}

impl ConnectionArgs {
    pub fn client_factory(&self) -> Result<client::DynamicClientFactory> {
        // Transports
        let t_tcp = transport::TransportFactory::try_new(self.connect_timeout.into())?;
        let t_tcp = Arc::new(t_tcp);
        let _t_opaque = transport::TransportFactory::new();

        // Clients
        let c_ssh = client::SshClientFactory::new(t_tcp.clone());
        let c_tcp = client::TcpClientFactory::new(t_tcp.clone());

        // Mapper
        let mapper = client::DynamicClientFactory::new().with(c_ssh).with(c_tcp);
        Ok(mapper)
    }
}
