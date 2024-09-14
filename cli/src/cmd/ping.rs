use std::time::Duration;

use astu_action::tcp::TcpClient;
use astu_action::Connect;
use astu_action::Ping;
use astu_util::id::Id;
use clap::Args;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::argetype::ConnectionArgs;
use crate::argetype::ResolutionArgs;

/// Connect to targets
#[derive(Debug, Args)]
pub struct PingArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,

    #[command(flatten)]
    connection_args: ConnectionArgs,
}

impl super::Run for PingArgs {
    async fn run(&self, id: Id) -> anyhow::Result<()> {
        eprintln!("Invocation ID: {id}");

        let _targets = self.resolution_args.clone().resolve();

        Ok(())
    }
}

async fn _ping(mut client: TcpClient, connect_timeout: Duration) {
    match client.connect(connect_timeout).await {
        Ok(_empty) => debug!("connect succeeded"),
        Err(error) => {
            error!(?error, "connect error");
            return;
        }
    }

    match client.ping().await {
        Ok(response) => info!(?response, "ping succeeded"),
        Err(error) => error!(?error, "ping error"),
    }
}
