use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;

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

/// Factory for building TCP clients.
#[derive(Debug, Clone)]
pub struct TcpClientFactory {
    transport: TransportFactoryImpl,
}

impl TcpClientFactory {
    #[must_use]
    pub fn new(transport: TransportFactoryImpl) -> Self {
        Self { transport }
    }
}

impl ClientFactory for TcpClientFactory {
    fn client(&self, target: &Target) -> Option<ClientImpl> {
        let client = match target.socket_addr() {
            Some(_addr) => TcpClient::new(self.transport.clone(), target),
            _other => return None,
        };
        Some(client.into())
    }
}

// Client ---------------------------------------------------------------------

/// TCP client.
pub struct TcpClient {
    transport: TransportFactoryImpl,
    target: Target,
    stream: Option<TcpStream>,
}

impl TcpClient {
    #[must_use]
    pub fn new(transport: TransportFactoryImpl, target: &Target) -> Self {
        Self {
            transport,
            target: target.to_owned(),
            stream: None,
        }
    }
}

#[async_trait]
impl Client for TcpClient {
    async fn connect(&mut self) -> Result<()> {
        if self.stream.is_some() {
            bail!("tcp stream is already connected");
        }

        let transport = self
            .transport
            .setup(&self.target)
            .await
            .context("failed connecting target transport")?;

        self.stream = match transport {
            Transport::Tcp(stream) => Some(stream),
            unsupported => bail!("unsupported TcpClient stream: {unsupported:?}"),
        };

        Ok(())
    }

    async fn ping(&mut self) -> Result<Vec<u8>> {
        let stream = self.stream.take().context("stream not connected")?;
        let mut reader = BufReader::new(stream);

        let mut output = Vec::new();
        reader.read_until(b'\n', &mut output).await?;
        let output = output.trim_ascii().to_owned();

        self.stream = Some(reader.into_inner());

        Ok(output)
    }

    async fn auth(&mut self, _auth_type: &AuthPayload) -> Result<()> {
        bail!("TcpClient::auth not supported");
    }

    async fn exec(&mut self, _command: &str) -> Result<ExecOutput> {
        bail!("TcpClient::exec not supported");
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::time::Duration;

    use rstest::rstest;

    use super::*;
    use crate::action::transport;

    #[rstest]
    #[case("10.0.0.54:22")]
    #[tokio::test]
    async fn works(#[case] input: &str) {
        let target = Target::from_str(input).unwrap();

        let timeout = Duration::from_secs(2);
        let transport = transport::tcp::TransportFactory::new(timeout).into();

        let mut client = TcpClient::new(transport, &target);
        client.connect().await.unwrap();
    }
}
