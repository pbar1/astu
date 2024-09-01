use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use kush_connect::ssh::SshClient;
use kush_connect::tcp::ReuseportTcpFactory;
use kush_connect::tcp::TcpFactoryAsync;
use kush_resolve::Resolve;
use kush_resolve::ResolveChain;
use kush_resolve::Target;
use tracing::debug;

/// Run commands on targets
#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Target query
    query: String,

    /// Command to run
    command: String,

    /// User to authenticate as
    #[arg(long, default_value = "root")]
    user: String,
}

#[async_trait::async_trait]
impl super::Run for ExecArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let resolvers = ResolveChain::try_default()?;
        let targets = resolvers.resolve(&self.query).await?;

        clowntown_ssh(targets).await?;

        Ok(())
    }
}

async fn clowntown_ssh(targets: impl IntoIterator<Item = Target>) -> anyhow::Result<()> {
    let tcp_factory = ReuseportTcpFactory::try_new()?;
    let tcp_factory = Arc::new(tcp_factory);

    let mut tasks = Vec::new();
    for target in targets {
        let tcp_factory = tcp_factory.clone();
        tasks.push(tokio::spawn(async move {
            let addr = match target {
                Target::Ipv4Addr(x) => SocketAddr::new(IpAddr::V4(x), 22),
                Target::Ipv6Addr(x) => SocketAddr::new(IpAddr::V6(x), 22),
                Target::SocketAddrV4(x) => SocketAddr::V4(x),
                Target::SocketAddrV6(x) => SocketAddr::V6(x),
                unsupported => {
                    debug!(target = %unsupported, "unsupported target type for exec");
                    return;
                }
            };

            let stream = tcp_factory
                .clone()
                .connect_timeout_async(&addr, Duration::from_secs(5))
                .await
                .unwrap();

            let mut ssh_client = SshClient::connect(stream).await.unwrap();

            ssh_client.auth_agent("nixos").await.unwrap();
            let output = ssh_client.exec("uname -a").await.unwrap();

            let _stdout = String::from_utf8_lossy(&output.stdout);
        }));
    }

    let _x = futures::future::join_all(tasks).await;

    Ok(())
}
