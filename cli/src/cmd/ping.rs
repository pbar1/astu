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
use astu_db::Db;
use astu_db::PingEntry;
use astu_db::SqliteDb;
use astu_resolve::Target;
use astu_util::combinator::AstuTryStreamExt;
use astu_util::id::Id;
use clap::Args;
use futures::StreamExt;
use futures::TryStreamExt;
use tracing::debug;
use tracing::warn;

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
        // FIXME: clowntown
        let db = SqliteDb::try_new("astu.db").await?;
        db.migrate().await?;

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

        // let db = targets
        //     .map(|t| client_factory.get_client(t))
        //     .flatten_err(|error| debug!(?error, "unable to get client"))
        //     .map(|c| ping(c))
        //     .buffer_unordered(self.connection_args.concurrency)
        //     .fold(db, save)
        //     .await;

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

async fn save(db: SqliteDb, result: Result<String>) -> SqliteDb {
    let entry = match result {
        Ok(message) => PingEntry {
            job_id: Vec::new(),
            target: "".into(),
            error: None,
            message: message.as_bytes().to_vec(),
        },
        Err(error) => PingEntry {
            job_id: Vec::new(),
            target: "".into(),
            error: Some(error.to_string()),
            message: "".into(),
        },
    };
    if let Err(error) = db.save_ping(&entry).await {
        warn!(?error, "unable to save ping entry");
    }
    db
}
