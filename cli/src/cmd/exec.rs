use std::time::Duration;

use astu_action::ssh::SshClient;
use astu_action::ssh::SshFactory;
use astu_action::Auth;
use astu_action::AuthType;
use astu_action::Connect;
use astu_action::Exec;
use astu_util::id::Id;
use clap::Args;
use futures::StreamExt;
use tracing::debug;
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

    /// Bind all connections to the same local address.
    ///
    /// This greatly increases the possible number of concurrent connections, at
    /// the cost of being unable to create more than one simultaneous connection
    /// to each remote address.
    #[arg(long)]
    reuseport: bool,

    /// Time in seconds to allow connection to complete.
    #[arg(long, default_value_t = 2)]
    connect_timeout: u64,
}

#[async_trait::async_trait]
impl super::Run for ExecArgs {
    async fn run(&self, id: Id) -> anyhow::Result<()> {
        eprintln!("Invocation ID: {id}");

        let targets = self.resolution_args.clone().resolve();

        // TODO: shlex join
        let command = self.command.join(" ");
        let user = Some(self.user.clone());

        // TODO: This block is hardcoded to SSH
        let ssh = match self.reuseport {
            true => SshFactory::reuseport(user.clone())?,
            false => SshFactory::regular(user.clone()),
        };
        let mut auths = Vec::new();
        if let Some(socket) = self.ssh_agent.clone() {
            auths.push(AuthType::SshAgent { socket });
        }
        let connect_timeout = Duration::from_secs(self.connect_timeout);

        let _stream = targets
            .inspect(|target| debug!(?target, "exec target"))
            .map(|target| ssh.get_client(target))
            .for_each_concurrent(0, |client| {
                exec(client, auths.clone(), command.clone(), connect_timeout)
            })
            .await;

        Ok(())
    }
}

async fn exec(
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
            (Ok(_), _) => authed = true,
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
