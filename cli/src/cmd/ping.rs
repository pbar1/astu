use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu_action::client::Client;
use astu_action::client::ClientFactory;
use astu_action::client::DynamicClientFactory;
use astu_action::Connect;
use astu_action::Ping;
use astu_db::Db;
use astu_db::DbImpl;
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
    #[clap(flatten)]
    resolution_args: ResolutionArgs,

    #[clap(flatten)]
    connection_args: ConnectionArgs,

    /// Database to use.
    #[clap(long)]
    db: String,
}

impl Run for PingArgs {
    async fn run(&self, id: Id) -> Result<()> {
        let db = DbImpl::try_new(&self.db)
            .await
            .context("unable to connect to a db")?;

        let targets = self.resolution_args.set().await?;
        let client_factory = self.connection_args.client_factory()?;

        let db = futures::stream::iter(targets)
            .filter_map(|t| {
                let client_factory = client_factory.clone();
                async move { client_factory.client(&t) }
            })
            .map(|c| ping(c))
            .buffer_unordered(self.connection_args.concurrency)
            .fold(
                db,
                |db, res| async move { save(db, res, &id.to_string()).await },
            )
            .await;

        Ok(())
    }
}

async fn ping2(
    target: Target,
    client_factory: DynamicClientFactory,
    job_id: &str,
    db: DbImpl,
) -> Option<()> {
    let client = client_factory.client(&target)?;

    // FIXME:

    Some(())
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

async fn save(db: DbImpl, result: Result<String>, job_id: &str) -> DbImpl {
    let entry = match result {
        Ok(message) => PingEntry {
            job_id: job_id.into(),
            target: "<todo>".into(),
            error: None,
            message: message.as_bytes().to_vec(),
        },
        Err(error) => PingEntry {
            job_id: job_id.into(),
            target: "<todo>".into(),
            error: Some(error.to_string()),
            message: "".into(),
        },
    };
    if let Err(error) = db.save_ping(&entry).await {
        warn!(?error, "unable to save ping entry");
    }
    db
}
