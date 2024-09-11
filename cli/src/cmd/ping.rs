use std::time::Duration;

use astu_action::tcp::TcpClient;
use astu_action::tcp::TcpClientFactory;
use astu_action::Connect;
use astu_action::Ping;
use astu_util::combinator::AstuTryStreamExt;
use clap::Args;
use futures::StreamExt;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::argetype::ResolutionArgs;

/// Connect to targets
#[derive(Debug, Args)]
pub struct PingArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,

    /// Port to connect to if not provided by target
    #[arg(long, default_value_t = 22)]
    port: u16,

    /// Bind all connections to the same local address.
    ///
    /// This greatly increases the possible number of concurrent connections, at
    /// the cost of being unable to create more than one simultaneous connection
    /// to each remote address.
    #[arg(long)]
    reuseport: bool,

    /// Time in seconds to allow connection to complete.
    #[arg(long, default_value_t = 2)]
    connect_timeout: u64,
}

#[async_trait::async_trait]
impl super::Run for PingArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let targets = self.resolution_args.clone().resolve();

        // TODO: This block is hardcoded to TCP
        let tcp = match self.reuseport {
            true => TcpClientFactory::reuseport(self.port)?,
            false => TcpClientFactory::regular(self.port),
        };
        let connect_timeout = Duration::from_secs(self.connect_timeout);

        let _stream = targets
            .inspect(|target| debug!(?target, "exec target"))
            .map(|target| tcp.get_client(target))
            .infallible(|error| error!(?error, "unable to get client"))
            .for_each_concurrent(None, |client| ping(client, connect_timeout))
            .await;

        Ok(())
    }
}

async fn ping(mut client: TcpClient, connect_timeout: Duration) {
    match client.connect(connect_timeout).await {
        Ok(_empty) => debug!("connect succeeded"),
        Err(error) => error!(?error, "connect error"),
    }

    match client.ping().await {
        Ok(response) => info!(?response, "ping succeeded"),
        Err(error) => error!(?error, "ping error"),
    }
}
