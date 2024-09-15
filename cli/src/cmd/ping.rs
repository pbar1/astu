use std::collections::BTreeMap;
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

        let (ok_count, err_count) = outcome.get_totals();
        let total = ok_count + err_count;
        let ok_pct = ok_count as f64 / total as f64 * 100f64;
        let err_pct = err_count as f64 / total as f64 * 100f64;
        println!("Success: {ok_count}/{total} ({ok_pct:.0}%)");
        println!("Failure: {err_count}/{total} ({err_pct:.0}%)");
        println!();
        println!("Success Frequency:");
        for (val, hits) in outcome.ok_freq {
            println!("{hits}: {val}");
        }
        println!();
        println!("Failure Frequency:");
        for (val, hits) in outcome.err_freq {
            println!("{hits}: {val}");
        }

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
    ok_freq: BTreeMap<String, u128>,
    err_freq: BTreeMap<String, u128>,
}

impl PingOutcome {
    fn get_totals(&self) -> (u128, u128) {
        let ok_count = self.ok_freq.values().sum();
        let err_count = self.err_freq.values().sum();
        (ok_count, err_count)
    }
}

async fn process_outcome(mut acc: PingOutcome, result: Result<String>) -> PingOutcome {
    match result {
        Ok(ok) => {
            acc.ok_freq.entry(ok).and_modify(|e| *e += 1).or_insert(1);
        }
        Err(err) => {
            acc.err_freq
                .entry(err.to_string())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
    }
    acc
}
