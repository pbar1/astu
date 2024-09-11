use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu_resolve::Target;
use astu_util::tcp::DefaultTcpFactory;
use astu_util::tcp::ReuseportTcpFactory;
use astu_util::tcp::TcpFactoryAsync;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;

use crate::Connect;
use crate::Ping;

// Factory --------------------------------------------------------------------

pub struct TcpClientFactory {
    factory: Arc<dyn TcpFactoryAsync + Send + Sync>,
    default_port: u16,
}

impl TcpClientFactory {
    pub fn new(factory: Arc<dyn TcpFactoryAsync + Send + Sync>, default_port: u16) -> Self {
        Self {
            factory,
            default_port,
        }
    }

    pub fn regular(default_port: u16) -> Self {
        let factory = Arc::new(DefaultTcpFactory);
        Self {
            factory,
            default_port,
        }
    }

    pub fn reuseport(default_port: u16) -> Result<Self> {
        let factory = ReuseportTcpFactory::try_new().map(Arc::new)?;
        Ok(Self {
            factory,
            default_port,
        })
    }

    pub fn get_client(&self, target: Target) -> Result<TcpClient> {
        let addr = match target {
            Target::IpAddr(ip) => SocketAddr::new(ip, self.default_port),
            Target::SocketAddr(addr) => addr,
            Target::Ssh { addr, user: _user } => addr,
            unsupported => bail!("unsupported target type for TcpClient: {unsupported}"),
        };
        let client = TcpClient::new(addr, self.factory.clone());
        Ok(client)
    }
}

// Client ---------------------------------------------------------------------

pub struct TcpClient {
    factory: Arc<dyn TcpFactoryAsync + Send + Sync>,
    addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl TcpClient {
    pub fn new(addr: SocketAddr, factory: Arc<dyn TcpFactoryAsync + Send + Sync>) -> Self {
        Self {
            addr,
            factory,
            stream: None,
        }
    }
}

#[async_trait::async_trait]
impl Connect for TcpClient {
    async fn connect(&mut self, timeout: Duration) -> Result<()> {
        if self.stream.is_some() {
            bail!("tcp stream is already connected");
        }

        let stream = self
            .factory
            .connect_timeout_async(&self.addr, timeout)
            .await?;
        self.stream = Some(stream);

        Ok(())
    }
}

#[async_trait::async_trait]
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

    use astu_util::tcp::DefaultTcpFactory;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("10.0.0.54:22", "SSH-2")]
    #[tokio::test]
    async fn ping_works(#[case] input: &str, #[case] should_contain: &str) {
        let addr = SocketAddr::from_str(input).unwrap();
        let timeout = Duration::from_secs(2);
        let factory: Arc<dyn TcpFactoryAsync + Send + Sync> = Arc::new(DefaultTcpFactory);

        let mut client = TcpClient::new(addr, factory);
        client.connect(timeout).await.unwrap();
        let output = client.ping().await.unwrap();

        assert!(output.contains(should_contain));
    }
}
