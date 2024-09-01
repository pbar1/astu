use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::io::AsyncWriteExt;

pub struct SshClient {
    session: russh::client::Handle<SshClientHandler>,
}

impl SshClient {
    async fn connect(stream: tokio::net::TcpStream) -> anyhow::Result<Self> {
        let config = Arc::new(russh::client::Config {
            inactivity_timeout: Some(Duration::from_secs(30)),
            ..<_>::default()
        });

        let handler = SshClientHandler;

        let session = russh::client::connect_stream(config, stream, handler).await?;

        Ok(Self { session })
    }

    async fn auth(
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

    async fn exec(&mut self, command: &str) -> anyhow::Result<ExecOutput> {
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

    async fn close(&mut self) -> anyhow::Result<()> {
        self.session
            .disconnect(russh::Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

struct SshClientHandler;

#[async_trait::async_trait]
impl russh::client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
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
        let addr = "127.0.0.1:2222".to_socket_addrs().unwrap().next().unwrap();
        let stream = tcp_factory
            .connect_timeout_async(&addr, Duration::from_secs(5))
            .await
            .unwrap();
        let _ssh_client = SshClient::connect(stream).await.unwrap();
    }
}
