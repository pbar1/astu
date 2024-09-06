use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use futures::StreamExt;
use kush_action::ssh::SshClient;
use kush_action::tcp::DefaultTcpFactory;
use kush_action::tcp::ReuseportTcpFactory;
use kush_action::tcp::TcpFactoryAsync;
use kush_action::Auth;
use kush_action::AuthType;
use kush_action::Connect;
use kush_action::Exec;
use tracing::debug;
use tracing::error;

use crate::argetype::ResolutionArgs;
use crate::mapper::ssh::SshMapper;

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
    async fn run(&self) -> anyhow::Result<()> {
        let targets = self.resolution_args.clone().resolve();

        // TODO: shlex join
        let command = self.command.join(" ");

        // TODO: This block is hardcoded to SSH
        let tcp: Arc<dyn TcpFactoryAsync + Send + Sync> = match self.reuseport {
            true => Arc::new(ReuseportTcpFactory::try_new()?),
            false => Arc::new(DefaultTcpFactory),
        };
        let mapper = SshMapper::new(tcp);
        let mut auths = Vec::new();
        auths.push(AuthType::User(self.user.clone()));
        if let Some(socket) = self.ssh_agent.clone() {
            auths.push(AuthType::SshAgent { socket });
        }
        let connect_timeout = Duration::from_secs(self.connect_timeout);

        let _stream = targets
            .inspect(|target| debug!(?target, "exec target"))
            .map(|target| mapper.get_client(target))
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

    for auth in auths {
        client.auth(&auth).await;
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