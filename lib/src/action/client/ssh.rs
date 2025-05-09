use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use russh::client::Handle;
use russh::keys::key::PrivateKeyWithHashAlg;
use russh::keys::HashAlg;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use tracing::error;

use crate::action::transport::Transport;
use crate::action::transport::TransportFactory;
use crate::action::transport::TransportFactoryImpl;
use crate::action::AuthPayload;
use crate::action::Client;
use crate::action::ClientFactory;
use crate::action::ClientImpl;
use crate::action::ExecOutput;
use crate::resolve::Target;

// Factory --------------------------------------------------------------------

/// Factory for building SSH clients.
#[derive(Debug, Clone)]
pub struct SshClientFactory {
    transport: TransportFactoryImpl,
}

impl SshClientFactory {
    #[must_use]
    pub fn new(transport: TransportFactoryImpl) -> Self {
        Self { transport }
    }
}

impl ClientFactory for SshClientFactory {
    fn client(&self, target: &Target) -> Option<ClientImpl> {
        let client = match target.socket_addr() {
            Some(_addr) => SshClient::new(self.transport.clone(), target),
            _other => return None,
        };
        Some(client.into())
    }
}

// Client ---------------------------------------------------------------------

/// SSH client.
pub struct SshClient {
    transport: TransportFactoryImpl,
    target: Target,
    session: Option<Handle<SshClientHandler>>,
    user: Option<String>,
}

impl SshClient {
    #[must_use]
    pub fn new(transport: TransportFactoryImpl, target: &Target) -> Self {
        Self {
            transport,
            target: target.to_owned(),
            session: None,
            user: None,
        }
    }

    /// # Errors
    ///
    /// If disconnect fails.
    pub async fn close(&mut self) -> Result<()> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };
        session
            .disconnect(russh::Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Client for SshClient {
    async fn connect(&mut self) -> anyhow::Result<()> {
        let config = Arc::new(russh::client::Config {
            inactivity_timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        });

        let handler = SshClientHandler::default();

        let transport = self
            .transport
            .setup(&self.target)
            .await
            .context("failed connecting target transport")?;

        let session = match transport {
            Transport::Tcp(stream) => {
                russh::client::connect_stream(config, stream, handler).await?
            }
            unsupported => bail!("unsupported TcpClient stream: {unsupported:?}"),
        };
        self.session = Some(session);

        Ok(())
    }

    async fn ping(&mut self) -> Result<Vec<u8>> {
        let _session = self.session.take().context("session not connected")?;
        Ok(Vec::new())
    }

    async fn auth(&mut self, auth_type: &AuthPayload) -> anyhow::Result<()> {
        match auth_type {
            AuthPayload::User(x) => self.auth_user(x).await,
            AuthPayload::Password(x) => self.auth_password(x).await,
            AuthPayload::SshKey(x) => self.auth_ssh_key(x).await,
            AuthPayload::SshCert { key, cert } => self.auth_ssh_cert(key, cert).await,
            AuthPayload::SshAgent { socket } => self.auth_ssh_agent(socket).await,
        }
    }

    async fn exec(&mut self, command: &str) -> anyhow::Result<ExecOutput> {
        self.exec_inner(command).await
    }
}

/// Helpers for [`Auth`]
impl SshClient {
    #[allow(clippy::unused_async)]
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
        if !authenticated.success() {
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
        let hash_alg = match key.algorithm() {
            russh::keys::Algorithm::Rsa { hash } => hash,
            _else => None,
        };
        let key = PrivateKeyWithHashAlg::new(Arc::new(key), hash_alg);

        let authenticated = session.authenticate_publickey(user, key).await?;
        if !authenticated.success() {
            bail!("ssh authentication failed");
        }

        Ok(())
    }

    async fn auth_ssh_cert(&mut self, private_key: &str, cert: &str) -> anyhow::Result<()> {
        let Some(ref mut session) = self.session else {
            bail!("no ssh session");
        };
        let Some(ref user) = self.user else {
            bail!("no ssh user");
        };

        let key = russh::keys::decode_secret_key(private_key, None)?;
        let cert = russh::keys::Certificate::from_openssh(cert)?;

        let authenticated = session
            .authenticate_openssh_cert(user, Arc::new(key), cert)
            .await?;
        if !authenticated.success() {
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
        let identities = agent.request_identities().await?;

        for key in identities {
            let fingerprint = key.fingerprint(HashAlg::default());
            let hash_alg = match key.algorithm() {
                russh::keys::Algorithm::Rsa { hash } => hash,
                _else => None,
            };
            let result = session
                .authenticate_publickey_with(user, key, hash_alg, &mut agent)
                .await;
            match result {
                Ok(auth_result) => {
                    if auth_result.success() {
                        return Ok(());
                    }
                    debug!(%user, key = %fingerprint, "ssh agent auth denied");
                }
                Err(error) => error!(?error, "ssh agent auth failed"),
            }
        }

        bail!("unable to authenticate with ssh agent");
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

#[derive(Debug, Default)]
struct SshClientHandler {
    server_banner: Option<String>,
}

impl russh::client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
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
    use std::str::FromStr;
    use std::time::Duration;

    use rstest::rstest;

    use super::*;
    use crate::action::transport;

    #[rstest]
    #[case("10.0.0.54:22", "nixos")]
    #[tokio::test]
    async fn works(#[case] input: &str, #[case] user: &str) {
        let target = Target::from_str(input).unwrap();

        let timeout = Duration::from_secs(2);
        let transport = transport::tcp::TransportFactory::new(timeout).into();

        let mut client = SshClient::new(transport, &target);
        client.connect().await.unwrap();

        // TODO: Use AuthType instead of direct functions
        let sshagent = std::env::var("SSH_AUTH_SOCK").unwrap();
        client.auth_user(user).await.unwrap();
        client.auth_ssh_agent(&sshagent).await.unwrap();

        let output = client.exec("uname -a").await.unwrap();
        assert_eq!(output.exit_status, 0);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("GNU/Linux"));
    }
}
