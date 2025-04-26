use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use astu::action::client::DynamicClientFactory;
use astu::action::Client;
use astu::action::ClientFactory;
use astu::db::Db;
use astu::db::DbImpl;
use astu::db::ResultEntry;
use astu::resolve::Target;
use astu::util::id::Id;
use astu::util::tokio::spawn_timeout;
use clap::Args;
use futures::StreamExt;
use tracing::instrument;
use tracing::warn;

use crate::args::ConnectionArgs;
use crate::args::ResolutionArgs;
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
        println!("Job ID: {id}");

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
) -> ResultEntry {
    // TODO: Maybe a better way to flatten
    let result = spawn_timeout(timeout, ping_inner(target.clone(), client_factory)).await;

    match result {
        Ok(Ok(message)) => ResultEntry {
            job_id: job_id.clone(),
            target: target.to_string(),
            error: None,
            exit_status: None,
            stdout: Some(message),
            stderr: None,
        },
        Ok(Err(error)) | Err(error) => ResultEntry {
            job_id: job_id.clone(),
            target: target.to_string(),
            error: Some(format!("{error:?}")),
            exit_status: None,
            stdout: None,
            stderr: None,
        },
    }
}

async fn ping_inner(target: Target, client_factory: DynamicClientFactory) -> Result<Vec<u8>> {
    let mut client = client_factory
        .client(&target)
        .context("failed getting client for target")?;

    client.connect().await.context("unable to connect")?;

    let output = client.ping().await.context("unable to ping")?;

    Ok(output)
}

async fn save(db: DbImpl, entry: ResultEntry) -> DbImpl {
    if let Err(error) = db.save(&entry).await {
        warn!(?error, ?entry, "failed saving entry to db");
    }
    db
}
