use std::time::Duration;

use astu_action::ssh::SshClient;
use astu_action::Auth;
use astu_action::AuthType;
use astu_action::Connect;
use astu_action::Exec;
use astu_util::id::Id;
use clap::Args;
use tracing::error;

use crate::argetype::ResolutionArgs;

/// Run commands on targets
#[derive(Debug, Args)]
pub struct ExecArgs {
    #[command(flatten)]
    resolution_args: ResolutionArgs,

    /// Command to run.
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,

    /// Remote user to authenticate as.
    #[arg(short = 'u', long, default_value = "root")]
    user: String,

    /// SSH agent socket to use.
    #[arg(long, env = "SSH_AUTH_SOCK")]
    ssh_agent: Option<String>,
}

impl super::Run for ExecArgs {
    async fn run(&self, id: Id) -> anyhow::Result<()> {
        eprintln!("Invocation ID: {id}");

        let _targets = self.resolution_args.clone().resolve();

        Ok(())
    }
}

async fn _exec(
    client: anyhow::Result<SshClient>,
    auths: Vec<AuthType>,
    command: String,
    timeout: Duration,
) {
    let mut client = match client {
        Ok(client) => client,
        Err(error) => {
            error!(?error, "failed to get client");
            return;
        }
    };

    if let Err(error) = client.connect(timeout).await {
        error!(?error, "failed to connect");
        return;
    };

    let mut authed = false;
    for auth in auths {
        match (client.auth(&auth).await, auth) {
            (Err(error), _) => error!(?error, "authentication failed"),
            (_, AuthType::User(_)) => continue,
            (Ok(()), _) => authed = true,
        }
    }
    if !authed {
        error!("all authentication attempts failed");
        return;
    }

    let output = match client.exec(&command).await {
        Ok(output) => output,
        Err(error) => {
            error!(?error, %command, "failed to exec");
            return;
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{stdout}");
}
