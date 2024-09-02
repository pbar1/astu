use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use futures::pin_mut;
use futures::StreamExt;
use kush_connect::tcp::DefaultTcpFactory;
use kush_connect::tcp::ReuseportTcpFactory;
use kush_connect::tcp::TcpFactoryAsync;
use kush_connect::Auth;
use kush_connect::AuthType;
use kush_connect::Connect;
use kush_connect::Exec;
use kush_resolve::ForwardResolveChain;
use kush_resolve::Resolve;
use kush_resolve::Target;

use crate::mapper::ssh::SshMapper;

/// Run commands on targets
#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Target query
    target: Target,

    /// Command to run
    command: String,

    /// User to authenticate as
    #[arg(short = 'u', long, default_value = "root")]
    user: String,

    /// SSH agent socket to use
    #[arg(long, env = "SSH_AUTH_SOCK")]
    ssh_agent: Option<String>,

    /// Bind all connections to the same local address
    #[arg(long)]
    reuseport: bool,
}

#[async_trait::async_trait]
impl super::Run for ExecArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let resolvers = ForwardResolveChain::try_default()?;
        let targets = resolvers.resolve(self.target.clone());
        pin_mut!(targets);

        // TODO: What to do about mapper being coded to SSH...
        let tcp: Arc<dyn TcpFactoryAsync + Send + Sync> = match self.reuseport {
            true => Arc::new(ReuseportTcpFactory::try_new()?),
            false => Arc::new(DefaultTcpFactory),
        };
        let mapper = SshMapper::new(tcp);

        let mut tasks = Vec::new();
        while let Some(target) = targets.next().await {
            if target.is_unknown() {
                continue;
            }
            let who = target.to_string();
            let Ok(mut client) = mapper.get_client(target) else {
                continue;
            };

            let mut auths = Vec::new();
            auths.push(AuthType::User(self.user.clone()));
            if let Some(socket) = self.ssh_agent.clone() {
                auths.push(AuthType::SshAgent { socket });
            }

            let command = self.command.clone();

            tasks.push(tokio::spawn(async move {
                let Ok(_) = client.connect(Duration::from_secs(30)).await else {
                    return;
                };
                for auth in auths {
                    let Ok(_) = client.auth(&auth).await else {
                        return;
                    };
                }
                let Ok(output) = client.exec(&command).await else {
                    return;
                };
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("{who}: {}", stdout);
            }));
        }

        let _x = futures::future::join_all(tasks).await;

        Ok(())
    }
}
