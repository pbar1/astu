use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu_resolve::Target;
use astu_util::tcp_stream::DefaultTcpStreamFactory;
use astu_util::tcp_stream::ReuseportTcpStreamFactory;
use astu_util::tcp_stream::TcpStreamFactory;
use ssh_key::Certificate;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use tracing::error;

use crate::Auth;
use crate::AuthType;
use crate::Connect;
use crate::Exec;
use crate::ExecOutput;

// Factory --------------------------------------------------------------------

pub struct SshClientFactory {
    tcp: Arc<dyn TcpStreamFactory + Send + Sync>,
    default_user: Option<String>,
}

impl SshClientFactory {
    pub fn new(tcp: Arc<dyn TcpStreamFactory + Send + Sync>) -> Self {
        Self {
            tcp,
            default_user: None,
        }
    }

    #[must_use] pub fn regular(default_user: Option<String>) -> Self {
        let tcp = Arc::new(DefaultTcpStreamFactory);
        let default_user = default_user.clone();
        Self { tcp, default_user }
    }

    pub fn reuseport(default_user: Option<String>) -> Result<Self> {
        let tcp = ReuseportTcpStreamFactory::try_new().map(Arc::new)?;
        let default_user = default_user.clone();
        Ok(Self { tcp, default_user })
    }

    pub fn get_client(&self, target: Target) -> Result<SshClient> {
        let (addr, user) = match target {
            Target::IpAddr(x) => (SocketAddr::new(x, 22), None),
            Target::SocketAddr(x) => (x, None),
            Target::Ssh { addr, user } => (addr, user),
            unsupported => bail!("unsupported ssh target: {unsupported}"),
        };
        let user = match user {
            Some(u) => Some(u),
            None => self.default_user.clone(),
        };
        Ok(SshClient::new(addr, self.tcp.clone(), user))
    }
}

// Client ---------------------------------------------------------------------

pub struct SshClient {
    addr: SocketAddr,
    tcp: Arc<dyn TcpStreamFactory + Send + Sync>,
    session: Option<russh::client::Handle<SshClientHandler>>,
    user: Option<String>,
}

impl SshClient {
    pub fn new(
        addr: SocketAddr,
        tcp: Arc<dyn TcpStreamFactory + Send + Sync>,
        user: Option<String>,
    ) -> Self {
        Self {
            addr,
            tcp,
            session: None,
            user,
        }
    }

    pub async fn close(&mut self) -> anyhow::Result<()> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };
        session
            .disconnect(russh::Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Connect for SshClient {
    async fn connect(&mut self, timeout: Duration) -> anyhow::Result<()> {
        let config = Arc::new(russh::client::Config {
            inactivity_timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        });

        let stream = self.tcp.connect_timeout(&self.addr, timeout).await?;

        let handler = SshClientHandler {
            server_banner: None,
        };

        let session = russh::client::connect_stream(config, stream, handler).await?;
        self.session = Some(session);

        Ok(())
    }
}

#[async_trait::async_trait]
impl Auth for SshClient {
    async fn auth(&mut self, auth_type: &AuthType) -> anyhow::Result<()> {
        match auth_type {
            AuthType::User(x) => self.auth_user(x).await,
            AuthType::Password(x) => self.auth_password(x).await,
            AuthType::SshKey(x) => self.auth_ssh_key(x).await,
            AuthType::SshCert { key, cert } => self.auth_ssh_cert(key, cert.clone()).await,
            AuthType::SshAgent { socket } => self.auth_ssh_agent(socket).await,
        }
    }
}

/// Helpers for [`Auth`]
impl SshClient {
    async fn auth_user(&mut self, user: &str) -> anyhow::Result<()> {
        if self.user.is_some() {
            bail!("ssh user is already set");
        }
        self.user = Some(user.to_owned());
        Ok(())
    }

    async fn auth_password(&mut self, password: &str) -> anyhow::Result<()> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };
        let Some(ref user) = self.user else {
            bail!("no ssh user");
        };

        let authenticated = session.authenticate_password(user, password).await?;
        if !authenticated {
            bail!("ssh authentication failed");
        }

        Ok(())
    }

    async fn auth_ssh_key(&mut self, private_key: &str) -> anyhow::Result<()> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };
        let Some(ref user) = self.user else {
            bail!("no ssh user");
        };

        let key = russh::keys::decode_secret_key(private_key, None)?;

        let authenticated = session.authenticate_publickey(user, Arc::new(key)).await?;
        if !authenticated {
            bail!("ssh authentication failed");
        }

        Ok(())
    }

    async fn auth_ssh_cert(&mut self, private_key: &str, cert: Certificate) -> anyhow::Result<()> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };
        let Some(ref user) = self.user else {
            bail!("no ssh user");
        };

        let key = russh::keys::decode_secret_key(private_key, None)?;

        let authenticated = session
            .authenticate_openssh_cert(user, Arc::new(key), cert)
            .await?;
        if !authenticated {
            bail!("ssh authentication failed");
        }

        Ok(())
    }

    /// Iterates through all identities found in SSH agent and returns on the
    /// first authentication success, or failure if exhausted before success.
    async fn auth_ssh_agent(&mut self, socket: &str) -> anyhow::Result<()> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };
        let Some(ref user) = self.user else {
            bail!("no ssh user");
        };

        let mut agent = russh::keys::agent::client::AgentClient::connect_uds(socket).await?;
        let mut result: Result<bool, _>;
        let identities = agent.request_identities().await?;

        for key in identities {
            let fingerprint = key.fingerprint();
            (agent, result) = session.authenticate_future(user, key, agent).await;
            match result {
                Ok(true) => return Ok(()),
                Ok(false) => debug!(%user, key = %fingerprint, "ssh agent auth denied"),
                Err(error) => error!(?error, "ssh agent auth failed"),
            }
        }

        bail!("unable to authenticate with ssh agent");
    }
}

#[async_trait::async_trait]
impl Exec for SshClient {
    async fn exec(&mut self, command: &str) -> anyhow::Result<ExecOutput> {
        self.exec_inner(command).await
    }
}

/// Helpers for [`Exec`]
impl SshClient {
    async fn exec_inner(&mut self, command: &str) -> anyhow::Result<ExecOutput> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };

        let mut channel = session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut code = None;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        loop {
            let Some(msg) = channel.wait().await else {
                break;
            };

            match msg {
                russh::ChannelMsg::Data { ref data } => {
                    stdout.write_all(data).await?;
                    stdout.flush().await?;
                }
                russh::ChannelMsg::ExtendedData { ref data, ext: 1 } => {
                    stderr.write_all(data).await?;
                    stderr.flush().await?;
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                    // cannot leave the loop immediately, there might still be
                    // more data to receive
                }
                _ => {}
            }
        }

        let exit_status = code.context("program did not exit cleanly")?;

        Ok(ExecOutput {
            exit_status,
            stdout,
            stderr,
        })
    }
}

// russh details --------------------------------------------------------------

#[derive(Debug)]
struct SshClientHandler {
    server_banner: Option<String>,
}

#[async_trait::async_trait]
impl russh::client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn auth_banner(
        &mut self,
        banner: &str,
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        self.server_banner = Some(banner.to_owned());
        Ok(())
    }
}

// Tests ----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::net::ToSocketAddrs;

    use super::*;

    #[tokio::test]
    async fn works() {
        let addr = "tec.lan:22".to_socket_addrs().unwrap().next().unwrap();
        let tcp = Arc::new(ReuseportTcpStreamFactory::try_new().unwrap());
        let user = Some("nixos".to_string());

        let mut client = SshClient::new(addr, tcp, user);

        client.connect(Duration::from_secs(2)).await.unwrap();

        let socket = std::env::var("SSH_AUTH_SOCK").unwrap();
        client.auth_ssh_agent(&socket).await.unwrap();

        let output = client.exec("uname -a").await.unwrap();
        assert_eq!(output.exit_status, 0);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("GNU/Linux"));
    }
}
