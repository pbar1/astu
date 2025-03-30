use std::future::Future;
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
use clap::Args;
use futures::StreamExt;

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
    #[clap(long, default_value = "astu.db")]
    db: String,

    /// Time to allow action to complete
    #[clap(long, default_value = "30s")]
    pub timeout: humantime::Duration,
}

impl Run for PingArgs {
    async fn run(&self, id: Id) -> Result<()> {
        let job_id = id.to_string();
        let timeout = self.timeout.into();

        let db = DbImpl::try_new(&self.db)
            .await
            .context("unable to connect to a db")?;

        let targets = self.resolution_args.set().await?;
        let client_factory = self.connection_args.client_factory()?;

        let _results: Vec<_> = futures::stream::iter(targets)
            .map(|target| {
                ping2(
                    target,
                    client_factory.clone(),
                    job_id.clone(),
                    db.clone(),
                    timeout,
                )
            })
            .buffer_unordered(self.connection_args.concurrency)
            .collect()
            .await;

        Ok(())
    }
}

async fn ping2(
    target: Target,
    client_factory: DynamicClientFactory,
    job_id: String,
    db: DbImpl,
    timeout: Duration,
) -> Result<()> {
    let result = spawn_timeout(timeout, ping2_inner(target.clone(), client_factory)).await;

    // TODO: Maybe a better way to flatten
    let entry = match result {
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
    };

    db.save_ping(&entry).await.context("failed to save to db")?;

    Ok(())
}

async fn ping2_inner(target: Target, client_factory: DynamicClientFactory) -> Result<()> {
    let mut client = client_factory
        .client(&target)
        .context("failed getting client for target")?;

    client.connect().await.context("unable to connect")?;

    Ok(())
}

// FIXME: Extract to reusable place
pub async fn spawn_timeout<T>(
    duration: Duration,
    future: impl Future<Output = T> + Send + 'static,
) -> anyhow::Result<T>
where
    T: Send + 'static,
{
    let task = tokio::spawn(future);
    let timeout = tokio::time::timeout(duration, task);
    let t = timeout
        .await
        .context("tokio task timed out")?
        .context("tokio task failed to join")?;
    Ok(t)
}
