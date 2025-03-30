use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use astu_action::client::DynamicClientFactory;
use astu_action::Client;
use astu_action::ClientFactory;
use astu_db::Db;
use astu_db::DbImpl;
use astu_db::PingEntry;
use astu_resolve::Target;
use astu_util::id::Id;
use astu_util::tokio::spawn_timeout;
use clap::Args;
use futures::StreamExt;
use tracing::instrument;
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

    /// Time to allow action to complete
    #[clap(long, default_value = "30s")]
    pub timeout: humantime::Duration,
}

impl Run for PingArgs {
    async fn run(&self, id: Id, db: DbImpl) -> Result<()> {
        let job_id = id.to_string();
        let timeout = self.timeout.into();

        let targets = self.resolution_args.set().await?;
        let client_factory = self.connection_args.client_factory()?;

        let _db = futures::stream::iter(targets)
            .map(|target| ping(target, client_factory.clone(), job_id.clone(), timeout))
            .buffer_unordered(self.connection_args.concurrency)
            .fold(db, save)
            .await;

        Ok(())
    }
}

#[instrument(skip_all, fields(%target))]
async fn ping(
    target: Target,
    client_factory: DynamicClientFactory,
    job_id: String,
    timeout: Duration,
) -> PingEntry {
    // TODO: Maybe a better way to flatten
    let result = spawn_timeout(timeout, ping_inner(target.clone(), client_factory)).await;
    match result {
        Ok(Ok(())) => PingEntry {
            job_id: job_id.to_owned(),
            target: target.to_string(),
            error: None,
        },
        Ok(Err(error)) => PingEntry {
            job_id: job_id.to_owned(),
            target: target.to_string(),
            error: Some(format!("{error:?}")),
        },
        Err(error) => PingEntry {
            job_id: job_id.to_owned(),
            target: target.to_string(),
            error: Some(format!("{error:?}")),
        },
    }
}

async fn ping_inner(target: Target, client_factory: DynamicClientFactory) -> Result<()> {
    let mut client = client_factory
        .client(&target)
        .context("failed getting client for target")?;

    client.connect().await.context("unable to connect")?;

    Ok(())
}

async fn save(db: DbImpl, entry: PingEntry) -> DbImpl {
    if let Err(error) = db.save_ping(&entry).await {
        warn!(?error, ?entry, "failed saving entry to db");
    }
    db
}
