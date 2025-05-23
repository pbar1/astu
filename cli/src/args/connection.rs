use anyhow::Result;
use astu::action::client;
use astu::action::transport;
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
        let t_tcp = transport::tcp_reuse::TransportFactory::try_new(self.connect_timeout.into())?;

        // Clients
        let _c_ssh = client::SshClientFactory::new(t_tcp.clone().into());
        let c_tcp = client::TcpClientFactory::new(t_tcp.clone().into());

        // Mapper
        let mapper = client::DynamicClientFactory::default()
            // .with(c_ssh)
            .with(c_tcp);
        Ok(mapper)
    }
}
