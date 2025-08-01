use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use astu::action::client::DynamicClientFactory;
use astu::action::AuthPayload;
use astu::action::Client;
use astu::action::ClientFactory;
use astu::action::ExecOutput;
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

use crate::cmd::Run;

/// Run commands on targets
#[derive(Debug, Args)]
pub struct ExecArgs {
    #[command(flatten)]
    resolution_args: crate::args::ResolutionArgs,

    #[command(flatten)]
    auth_args: crate::args::AuthArgs,

    #[command(flatten)]
    action_args: crate::args::ActionArgs,

    /// Command to run.
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

impl Run for ExecArgs {
    async fn run(&self, id: Id, db: DbImpl) -> Result<()> {
        let job_id = id.to_string();
        let timeout = self.action_args.timeout.into();
        let command = self.command.join(" "); // TODO: shlex join

        let targets = self.resolution_args.set().await?;
        let client_factory = self.action_args.client_factory()?;

        // TODO: move auth to own arg group
        let mut auths = Vec::new();
        auths.push(AuthPayload::User(self.auth_args.user.clone()));
        if let Some(socket) = &self.auth_args.ssh_agent {
            auths.push(AuthPayload::SshAgent {
                socket: socket.to_string(),
            });
        }

        let _db = futures::stream::iter(targets)
            .map(|target| {
                exec(
                    target,
                    client_factory.clone(),
                    job_id.clone(),
                    timeout,
                    command.clone(),
                    auths.clone(),
                )
            })
            .buffer_unordered(self.action_args.concurrency)
            .fold(db, save)
            .await;

        Ok(())
    }
}

#[instrument(skip_all, fields(%target))]
async fn exec(
    target: Target,
    client_factory: DynamicClientFactory,
    job_id: String,
    timeout: Duration,
    command: String,
    auths: Vec<AuthPayload>,
) -> ResultEntry {
    // TODO: Maybe a better way to flatten
    let result = spawn_timeout(
        timeout,
        exec_inner(target.clone(), client_factory, command, auths),
    )
    .await;

    match result {
        Ok(Ok(output)) => ResultEntry {
            job_id: job_id.clone(),
            target: target.to_string(),
            error: None,
            exit_status: Some(output.exit_status),
            stdout: Some(output.stdout),
            stderr: Some(output.stderr),
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

async fn exec_inner(
    target: Target,
    client_factory: DynamicClientFactory,
    command: String,
    auths: Vec<AuthPayload>,
) -> Result<ExecOutput> {
    let mut client = client_factory
        .client(&target)
        .context("failed getting client for target")?;

    client.connect().await.context("unable to connect")?;

    // TODO: clowntown
    for auth in auths {
        client.auth(&auth).await?;
    }

    let output = client
        .exec(&command)
        .await
        .context("unable to run command")?;

    Ok(output)
}

async fn save(db: DbImpl, entry: ResultEntry) -> DbImpl {
    if let Err(error) = db.save(&entry).await {
        warn!(?error, ?entry, "failed saving entry to db");
    }
    db
}
