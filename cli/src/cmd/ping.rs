use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::bail;
use anyhow::Result;
use astu_action::client::Client;
use astu_action::client::ClientFactory;
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

        let targets = self.resolution_args.set().await?;
        let client_factory = self.connection_args.client_factory()?;

        let db = futures::stream::iter(targets)
            .filter_map(|t| {
                let client_factory = client_factory.clone();
                async move { client_factory.client(&t) }
            })
            .map(|c| ping(c))
            .buffer_unordered(self.connection_args.concurrency)
            .fold(db, save)
            .await;

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
