use std::sync::Arc;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;

use crate::transport::Transport;
use crate::transport::TransportFactory;
use crate::Connect;
use crate::Ping;

pub struct TcpClient {
    transport: Arc<dyn TransportFactory + Send + Sync>,
    target: Target,
    stream: Option<TcpStream>,
}

impl TcpClient {
    pub fn new(transport: Arc<dyn TransportFactory + Send + Sync>, target: &Target) -> Self {
        Self {
            transport,
            target: target.to_owned(),
            stream: None,
        }
    }
}

#[async_trait]
impl Connect for TcpClient {
    async fn connect(&mut self) -> Result<()> {
        if self.stream.is_some() {
            bail!("tcp stream is already connected");
        }

        let transport = self
            .transport
            .connect(&self.target)
            .await
            .context("failed connecting target transport")?;

        self.stream = match transport {
            Transport::Tcp(stream) => Some(stream),
            unsupported => bail!("unsupported TcpClient stream: {unsupported:?}"),
        };

        Ok(())
    }
}

#[async_trait]
impl Ping for TcpClient {
    async fn ping(mut self) -> Result<String> {
        let stream = self.stream.take().context("unable to take tcp stream")?;
        let mut reader = BufReader::new(stream);

        let mut output: String = String::new();
        reader.read_line(&mut output).await?;
        let output = output.trim().to_owned();

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::time::Duration;

    use astu_resolve::Target;
    use rstest::rstest;

    use super::*;
    use crate::transport::TcpTransportFactory;

    #[rstest]
    #[case("10.0.0.54:22", "SSH-2")]
    #[tokio::test]
    async fn works(#[case] input: &str, #[case] should_contain: &str) {
        let target = Target::from_str(input).unwrap();

        let timeout = Duration::from_secs(2);
        let factory: Arc<dyn TransportFactory + Send + Sync> =
            Arc::new(TcpTransportFactory::new(timeout));

        let mut client = TcpClient::new(factory, &target);
        client.connect().await.unwrap();
        let output = client.ping().await.unwrap();

        assert!(output.contains(should_contain));
    }
}
