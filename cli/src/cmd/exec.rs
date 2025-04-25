use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use astu::action::client::DynamicClientFactory;
use astu::action::AuthType;
use astu::action::Client;
use astu::action::ClientFactory;
use astu::action::ExecOutput;
use astu::db::Db;
use astu::db::DbImpl;
use astu::db::ExecEntry;
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

/// Run commands on targets
#[derive(Debug, Args)]
pub struct ExecArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,

    #[command(flatten)]
    connection_args: ConnectionArgs,

    /// Command to run.
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,

    /// Remote user to authenticate as.
    #[arg(short = 'u', long, default_value = "root")]
    user: String,

    /// SSH agent socket to use.
    #[arg(long, env = "SSH_AUTH_SOCK")]
    ssh_agent: Option<String>,

    /// Time to allow action to complete
    #[clap(long, default_value = "30s")]
    pub timeout: humantime::Duration,
}

impl Run for ExecArgs {
    async fn run(&self, id: Id, db: DbImpl) -> Result<()> {
        let job_id = id.to_string();
        let timeout = self.timeout.into();
        let command = self.command.join(" "); // TODO: shlex join

        let targets = self.resolution_args.set().await?;
        let client_factory = self.connection_args.client_factory()?;

        // TODO: move auth to own arg group
        let mut auths = Vec::new();
        auths.push(AuthType::User(self.user.clone()));
        if let Some(socket) = &self.ssh_agent {
            auths.push(AuthType::SshAgent {
                socket: socket.to_owned(),
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
            .buffer_unordered(self.connection_args.concurrency)
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
    auths: Vec<AuthType>,
) -> ExecEntry {
    // TODO: Maybe a better way to flatten
    let result = spawn_timeout(
        timeout,
        exec_inner(target.clone(), client_factory, command, auths),
    )
    .await;

    match result {
        Ok(Ok(output)) => ExecEntry {
            job_id: job_id.clone(),
            target: target.to_string(),
            error: None,
            exit_status: Some(output.exit_status),
            stdout: Some(output.stdout),
            stderr: Some(output.stderr),
        },
        Ok(Err(error)) | Err(error) => ExecEntry {
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
    auths: Vec<AuthType>,
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

async fn save(db: DbImpl, entry: ExecEntry) -> DbImpl {
    if let Err(error) = db.save_exec(&entry).await {
        warn!(?error, ?entry, "failed saving entry to db");
    }
    db
}
