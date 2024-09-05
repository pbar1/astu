use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use futures::StreamExt;
use futures::TryStreamExt;
use kush_connect::ssh::SshClient;
use kush_connect::tcp::DefaultTcpFactory;
use kush_connect::tcp::ReuseportTcpFactory;
use kush_connect::tcp::TcpFactoryAsync;
use kush_connect::Auth;
use kush_connect::AuthType;
use kush_connect::Connect;
use kush_connect::Exec;
use tracing::debug;

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
            .inspect(|target| debug!(?target, "exec"))
            .map(|target| mapper.get_client(target))
            .try_for_each_concurrent(0, |client| {
                exec(client, auths.clone(), command.clone(), connect_timeout)
            })
            .await;

        Ok(())
    }
}

async fn exec(
    mut client: SshClient,
    auths: Vec<AuthType>,
    command: String,
    timeout: Duration,
) -> anyhow::Result<()> {
    client.connect(timeout).await?;

    for auth in auths {
        if let Ok(_) = client.auth(&auth).await {
            break;
        };
    }

    let output = client.exec(&command).await?;
    let _stdout = String::from_utf8_lossy(&output.stdout);

    Ok(())
}

// async fn process_targets(targets: impl Stream<Item = Target>) {
//     pin_mut!(targets);
//     while let Some(target) = targets.next().await {
//         println!("{target}");
//     }
//     let command = self.command.join(" ");

//     // TODO: What to do about mapper being coded to SSH...
//     let tcp: Arc<dyn TcpFactoryAsync + Send + Sync> = match self.reuseport {
//         true => Arc::new(ReuseportTcpFactory::try_new()?),
//         false => Arc::new(DefaultTcpFactory),
//     };
//     let mapper = SshMapper::new(tcp);

//     let mut tasks = Vec::new();
//     while let Some(target) = targets.next().await {
//         if target.is_unknown() {
//             continue;
//         }
//         let who = target.to_string();
//         let Ok(mut client) = mapper.get_client(target) else {
//             continue;
//         };

//         let mut auths = Vec::new();
//         auths.push(AuthType::User(self.user.clone()));
//         if let Some(socket) = self.ssh_agent.clone() {
//             auths.push(AuthType::SshAgent { socket });
//         }

//         let command = command.clone();
//         let connect_timeout = Duration::from_secs(self.connect_timeout);

//         tasks.push(tokio::spawn(async move {
//             let Ok(_) = client.connect(connect_timeout).await else {
//                 return;
//             };
//             for auth in auths {
//                 let Ok(_) = client.auth(&auth).await else {
//                     return;
//                 };
//             }
//             let Ok(output) = client.exec(&command).await else {
//                 return;
//             };
//             let stdout = String::from_utf8_lossy(&output.stdout);
//             println!("{who}: {}", stdout);
//         }));
//     }

//     let _x = futures::future::join_all(tasks).await;

//     Ok(())
// }
