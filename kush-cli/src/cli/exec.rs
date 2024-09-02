use std::collections::BTreeSet;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use futures::pin_mut;
use futures::StreamExt;
use kush_connect::ssh::SshClient;
use kush_connect::tcp::ReuseportTcpFactory;
use kush_connect::tcp::TcpFactoryAsync;
use kush_connect::Auth;
use kush_connect::AuthType;
use kush_connect::Connect;
use kush_connect::Exec;
use kush_resolve::ForwardResolveChain;
use kush_resolve::Resolve;
use kush_resolve::Target;
use tracing::debug;

use crate::mapper::ssh::SshMapper;

/// Run commands on targets
#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Target query
    query: Target,

    /// Command to run
    command: String,

    /// User to authenticate as
    #[arg(long, default_value = "root")]
    user: String,

    /// SSH agent socket to use
    #[arg(long, env = "SSH_AUTH_SOCK")]
    ssh_agent: Option<String>,
}

#[async_trait::async_trait]
impl super::Run for ExecArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let resolvers = ForwardResolveChain::try_default()?;
        let targets = resolvers.resolve(self.query.clone());
        pin_mut!(targets);

        // TODO: What to do about mapper being coded to SSH...
        let tcp = ReuseportTcpFactory::try_new()?;
        let mapper = SshMapper::new(Arc::new(tcp));

        let mut tasks = Vec::new();
        while let Some(target) = targets.next().await {
            if target.is_unknown() {
                continue;
            }
            let who = target.to_string();
            let Ok(mut client) = mapper.get_client(target) else {
                continue;
            };
            let user = AuthType::User(self.user.clone());
            let agent = AuthType::SshAgent {
                socket: self.ssh_agent.clone().unwrap_or("".to_string()),
            };
            let command = self.command.clone();
            tasks.push(tokio::spawn(async move {
                client.connect(Duration::from_secs(30)).await.unwrap();
                // FIXME: Send of vec of AuthType, would also support partial auth
                client.auth(&user).await.unwrap();
                client.auth(&agent).await.unwrap();
                let output = client.exec(&command).await.unwrap();
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("{who}: {}", stdout)
            }));
        }

        let _x = futures::future::join_all(tasks).await;

        Ok(())
    }
}

// async fn clowntown_ssh(targets: impl IntoIterator<Item = Target>) ->
// anyhow::Result<()> {     let tcp_factory = ReuseportTcpFactory::try_new()?;
//     let tcp_factory = Arc::new(tcp_factory);

//     let mut tasks = Vec::new();
//     for target in targets {
//         let tcp_factory = tcp_factory.clone();
//         tasks.push(tokio::spawn(async move {
//             let addr = match target {
//                 Target::Ipv4Addr(x) => SocketAddr::new(IpAddr::V4(x), 22),
//                 Target::Ipv6Addr(x) => SocketAddr::new(IpAddr::V6(x), 22),
//                 Target::SocketAddrV4(x) => SocketAddr::V4(x),
//                 Target::SocketAddrV6(x) => SocketAddr::V6(x),
//                 unsupported => {
//                     debug!(target = %unsupported, "unsupported target type
// for exec");                     return;
//                 }
//             };

//             let stream = tcp_factory
//                 .clone()
//                 .connect_timeout_async(&addr, Duration::from_secs(5))
//                 .await
//                 .unwrap();

//             let mut ssh_client = SshClient::connect(stream).await.unwrap();

//             ssh_client.auth_agent("nixos").await.unwrap();
//             let output = ssh_client.exec("uname -a").await.unwrap();

//             let _stdout = String::from_utf8_lossy(&output.stdout);
//         }));
//     }

//     let _x = futures::future::join_all(tasks).await;

//     Ok(())
// }
