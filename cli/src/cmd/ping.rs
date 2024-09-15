use std::collections::BTreeSet;
use std::time::Duration;

use anyhow::bail;
use anyhow::Result;
use astu_action::client::Client;
use astu_action::client::ClientFactory;
use astu_action::ssh::SshClientFactory;
use astu_action::tcp::TcpClientFactory;
use astu_action::Connect;
use astu_action::Ping;
use astu_util::combinator::AstuTryStreamExt;
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

        let connect_timeout = Duration::from_secs(self.connection_args.connect_timeout);

        let stream_factory = self.connection_args.tcp_stream_factory()?;
        let tcp_factory = TcpClientFactory::new(
            stream_factory.clone(),
            self.connection_args.port,
            connect_timeout,
        );
        let ssh_factory = SshClientFactory::new(stream_factory.clone(), connect_timeout);
        let client_factory = ClientFactory::new(tcp_factory, ssh_factory);

        let outcome = targets
            .map(|t| client_factory.get_client(t))
            .dropspect_err(|error| debug!(?error, "unable to get client"))
            .map(|c| ping(c))
            .buffer_unordered(self.connection_args.concurrency)
            .fold(PingOutcome::default(), process_outcome)
            .await;

        dbg!(outcome);

        Ok(())
    }
}

// TODO: Consider moving this into the Ping trait behavior
async fn ping(client: Client) -> Result<String> {
    match client {
        Client::Tcp(mut tcp) => {
            tcp.connect().await?;
            tcp.ping().await
        }
        _unsupported => bail!("clowntown ping only supported for tcp"),
    }
}

#[derive(Debug, Default)]
struct PingOutcome {
    ok_count: u128,
    err_count: u128,
    ok_freq: BTreeSet<String>,
    err_freq: BTreeSet<String>,
}

async fn process_outcome(mut acc: PingOutcome, result: Result<String>) -> PingOutcome {
    match result {
        Ok(ok) => {
            acc.ok_count += 1;
            acc.ok_freq.insert(ok);
        }
        Err(err) => {
            acc.err_count += 1;
            acc.err_freq.insert(err.to_string());
        }
    }
    acc
}
