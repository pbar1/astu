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
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;

use crate::Connect;
use crate::Ping;

// Factory --------------------------------------------------------------------

pub struct TcpClientFactory {
    tcp: Arc<dyn TcpStreamFactory + Send + Sync>,
    default_port: u16,
}

impl TcpClientFactory {
    pub fn new(tcp: Arc<dyn TcpStreamFactory + Send + Sync>, default_port: u16) -> Self {
        Self { tcp, default_port }
    }

    #[must_use] pub fn regular(default_port: u16) -> Self {
        let factory = Arc::new(DefaultTcpStreamFactory);
        Self {
            tcp: factory,
            default_port,
        }
    }

    pub fn reuseport(default_port: u16) -> Result<Self> {
        let factory = ReuseportTcpStreamFactory::try_new().map(Arc::new)?;
        Ok(Self {
            tcp: factory,
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
        let client = TcpClient::new(addr, self.tcp.clone());
        Ok(client)
    }
}

// Client ---------------------------------------------------------------------

pub struct TcpClient {
    tcp: Arc<dyn TcpStreamFactory + Send + Sync>,
    addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl TcpClient {
    pub fn new(addr: SocketAddr, tcp: Arc<dyn TcpStreamFactory + Send + Sync>) -> Self {
        Self {
            addr,
            tcp,
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

        let stream = self.tcp.connect_timeout(&self.addr, timeout).await?;
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

    use astu_util::tcp_stream::DefaultTcpStreamFactory;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("10.0.0.54:22", "SSH-2")]
    #[tokio::test]
    async fn ping_works(#[case] input: &str, #[case] should_contain: &str) {
        let addr = SocketAddr::from_str(input).unwrap();
        let timeout = Duration::from_secs(2);
        let factory: Arc<dyn TcpStreamFactory + Send + Sync> = Arc::new(DefaultTcpStreamFactory);

        let mut client = TcpClient::new(addr, factory);
        client.connect(timeout).await.unwrap();
        let output = client.ping().await.unwrap();

        assert!(output.contains(should_contain));
    }
}
