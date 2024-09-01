use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use tracing::error;

pub struct SshClient {
    session: russh::client::Handle<SshClientHandler>,
}

impl SshClient {
    pub async fn connect(stream: tokio::net::TcpStream) -> anyhow::Result<Self> {
        let config = Arc::new(russh::client::Config {
            inactivity_timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        });

        let handler = SshClientHandler {
            server_banner: None,
        };

        let session = russh::client::connect_stream(config, stream, handler).await?;

        Ok(Self { session })
    }

    pub async fn auth_key(
        &mut self,
        key_path: impl AsRef<Path>,
        user: impl Into<String>,
    ) -> anyhow::Result<()> {
        let key_pair = russh::keys::load_secret_key(key_path, None)?;

        let auth_res = self
            .session
            .authenticate_publickey(user, Arc::new(key_pair))
            .await?;

        if !auth_res {
            anyhow::bail!("Authentication failed");
        }

        Ok(())
    }

    pub async fn auth_agent(&mut self, user: &str) -> anyhow::Result<()> {
        let mut agent = russh::keys::agent::client::AgentClient::connect_env().await?;
        let mut result: Result<bool, _>;
        let identities = agent.request_identities().await?;

        for key in identities {
            let fingerprint = key.fingerprint();
            (agent, result) = self.session.authenticate_future(user, key, agent).await;
            match result {
                Ok(true) => return Ok(()),
                Ok(false) => debug!(%user, key = %fingerprint, "ssh agent auth denied"),
                Err(error) => error!(?error, "ssh agent auth failed"),
            }
        }

        bail!("unable to authenticate with ssh agent");
    }

    pub async fn exec(&mut self, command: &str) -> anyhow::Result<ExecOutput> {
        let mut channel = self.session.channel_open_session().await?;
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

    pub async fn close(&mut self) -> anyhow::Result<()> {
        self.session
            .disconnect(russh::Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

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

pub struct ExecOutput {
    pub exit_status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use std::net::ToSocketAddrs;

    use super::*;
    use crate::tcp::ReuseportTcpFactory;
    use crate::tcp::TcpFactoryAsync;

    #[tokio::test]
    async fn works() {
        let tcp_factory = ReuseportTcpFactory::try_new().unwrap();
        let addr = "tec.lan:22".to_socket_addrs().unwrap().next().unwrap();
        let stream = tcp_factory
            .connect_timeout_async(&addr, Duration::from_secs(5))
            .await
            .unwrap();
        let mut ssh_client = SshClient::connect(stream).await.unwrap();
        ssh_client.auth_agent("nixos").await.unwrap();
        let output = ssh_client.exec("uname -a").await.unwrap();
        assert_eq!(output.exit_status, 0);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("GNU/Linux"));
    }
}
