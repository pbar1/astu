use std::time::Duration;

use anyhow::Result;
use astu_action::client::ClientFactory;
use astu_action::ssh::SshClientFactory;
use astu_action::tcp::TcpClientFactory;
use astu_action::Connect;
use astu_action::Ping;
use astu_util::id::Id;
use clap::Args;
use futures::StreamExt;
use tracing::debug;

use crate::argetype::ConnectionArgs;
use crate::argetype::ResolutionArgs;
use crate::cmd::Run;

/// Connect to targets
#[derive(Debug, Args)]
pub struct PingArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,

    #[command(flatten)]
    connection_args: ConnectionArgs,
}

impl Run for PingArgs {
    async fn run(&self, _id: Id) -> Result<()> {
        let targets = self.resolution_args.clone().resolve();

        let stream_factory = self.connection_args.tcp_stream_factory()?;
        let tcp_factory = TcpClientFactory::new(stream_factory.clone(), self.connection_args.port);
        let ssh_factory = SshClientFactory::new(stream_factory.clone());
        let client_factory = ClientFactory::new(tcp_factory, ssh_factory);

        let connect_timeout = Duration::from_secs(self.connection_args.connect_timeout);

        targets
            .filter_map(|target| async {
                match client_factory.get_tcp_client(target) {
                    Ok(client) => Some(client),
                    Err(error) => {
                        debug!(?error, "unable to get client");
                        None
                    }
                }
            })
            .for_each_concurrent(None, |mut client| async move {
                match client.connect(connect_timeout.clone()).await {
                    Ok(_) => debug!("connect succeeded"),
                    Err(error) => {
                        debug!(?error, "connect failed");
                        return;
                    }
                }
                match client.ping().await {
                    Ok(output) => println!("{output}"),
                    Err(error) => debug!(?error, "ping failed"),
                }
            })
            .await;

        Ok(())
    }
}
