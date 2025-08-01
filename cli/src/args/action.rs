use anyhow::Result;
use astu::action::client;
use astu::action::transport;
use clap::Args;

const HEADING: Option<&str> = Some("Action Options");

/// Arguments for action execution.
#[derive(Debug, Args, Clone)]
pub struct ActionArgs {
    /// Number of actions to process at once.
    #[clap(long, default_value_t = 500, help_heading = HEADING)]
    pub concurrency: usize,

    /// Confirm target count
    #[clap(long, help_heading = HEADING)]
    pub confirm: Option<usize>,

    /// Time to allow each action to complete.
    #[clap(long, default_value = "30s", help_heading = HEADING)]
    pub timeout: humantime::Duration,
}

impl ActionArgs {
    pub fn client_factory(&self) -> Result<client::DynamicClientFactory> {
        // Transports
        let t_tcp = transport::tcp_reuse::TransportFactory::try_new(self.timeout.into())?;

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
