use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use async_stream::try_stream;
use futures::stream::BoxStream;
use futures::StreamExt;
use hickory_resolver::proto::rr::rdata::PTR;
use hickory_resolver::Name;
use hickory_resolver::TokioResolver;

use crate::resolve::Resolve;
use crate::resolve::Target;

/// Resolves DNS queries - both forward and reverse - into targets.
#[derive(Debug, Clone)]
pub struct DnsResolver {
    dns: TokioResolver,
    forward: bool,
    reverse: bool,
}

impl Resolve for DnsResolver {
    fn resolve_fallible(&self, target: Target) -> BoxStream<Result<Target>> {
        let fwd = self.forward;
        let rev = self.reverse;
        match target {
            Target::Domain { name, port } if fwd => self.resolve_domain(name, port),
            Target::IpAddr(ip) if rev => self.resolve_ip(ip, None),
            Target::SocketAddr(sock) if rev => self.resolve_ip(sock.ip(), Some(sock.port())),
            _unsupported => futures::stream::empty().boxed(),
        }
    }
}

impl DnsResolver {
    /// Creates a DNS resolver using the system DNS config. Forward resolution
    /// is enabled by default, while reverse resolution is disabled.
    ///
    /// # Errors
    ///
    /// - If the system resolver config fails to build.
    pub fn try_new() -> Result<Self> {
        // TODO: Use `Ipv4AndIpv6` strategy instead of the default `Ipv4thenIpv6`
        let dns = TokioResolver::builder_tokio()?.build();
        Ok(Self {
            dns,
            forward: true,
            reverse: false,
        })
    }

    /// Set forward lookup.
    #[must_use]
    pub fn with_forward(mut self, enable: bool) -> Self {
        self.forward = enable;
        self
    }

    /// Set reverse lookup.
    #[must_use]
    pub fn with_reverse(mut self, enable: bool) -> Self {
        self.reverse = enable;
        self
    }

    /// Forward DNS resolution
    fn resolve_domain(&self, name: Name, port: Option<u16>) -> BoxStream<Result<Target>> {
        try_stream! {
            let ips = self.dns.lookup_ip(name).await?;
            for ip in ips {
                yield match port {
                    Some(port) => Target::from(SocketAddr::new(ip, port)),
                    None => Target::from(ip),
                }
            }
        }
        .boxed()
    }

    /// Reverse DNS resolution
    fn resolve_ip(&self, ip: IpAddr, port: Option<u16>) -> BoxStream<Result<Target>> {
        try_stream! {
            let names = self.dns.reverse_lookup(ip).await?;
            for ptr in names {
                let name = remove_trailing_dot(&ptr)?;
                yield Target::Domain { name, port }
            }
        }
        .boxed()
    }
}

fn remove_trailing_dot(ptr: &PTR) -> Result<Name> {
    let name = ptr.0.to_string();
    let name = name
        .strip_suffix('.')
        .context("unable to strip trailing dot")?;
    let name = Name::from_str(name)?;
    Ok(name)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::resolve::ResolveExt;

    #[rstest]
    #[case("localhost")]
    #[case("google.com")]
    #[case("google.com.")]
    #[case("127.0.0.1")]
    #[case("127.0.0.1:22")]
    #[tokio::test]
    async fn resolve_works(#[case] query: &str) {
        let target = Target::from_str(query).unwrap();
        let resolver = DnsResolver::try_new().unwrap().with_reverse(true);
        let targets = resolver.resolve_set(target).await;
        assert!(!targets.is_empty());
    }
}
