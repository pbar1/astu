use std::net::IpAddr;
use std::net::SocketAddr;

use anyhow::bail;
use futures::FutureExt;
use futures::StreamExt;
use hickory_resolver::Name;
use hickory_resolver::TokioAsyncResolver;

use crate::Resolve;
use crate::ResolveResult;
use crate::Target;
use crate::TargetStream;

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl Resolve for DnsResolver {
    fn resolve(&self, target: Target) -> ResolveResult {
        let stream = match target {
            // Forward
            Target::Domain { name, port: None } => self.resolve_ip(name),
            Target::Domain {
                name,
                port: Some(p),
            } => self.resolve_sock(name, p),

            // Reverse
            Target::IpAddr(ip) => self.resolve_domain(ip),
            Target::SocketAddr(sock) => self.resolve_domain_port(sock),

            // Unsupported
            unsupported => bail!("DnsResolver: unsupported target: {unsupported}"),
        };

        Ok(stream)
    }
}

impl DnsResolver {
    pub fn system() -> anyhow::Result<Self> {
        let (config, options) = hickory_resolver::system_conf::read_system_conf()?;
        let resolver = TokioAsyncResolver::tokio(config, options);
        Ok(Self { resolver })
    }
}

// Forward resolution
impl DnsResolver {
    // TODO: Default IP lookup strategy is `Ipv4thenIpv6`. Consider
    // changing it to `Ipv4AndIpv6` to gather all possible IPs.

    fn resolve_ip(&self, name: Name) -> TargetStream {
        self.resolver
            .lookup_ip(name.clone())
            .into_stream()
            .map(futures::stream::iter)
            .flatten()
            .map(futures::stream::iter)
            .flatten()
            .map(Target::from)
            .boxed()
    }

    fn resolve_sock(&self, name: Name, port: u16) -> TargetStream {
        let iter = self
            .resolver
            .lookup_ip(name)
            .await
            .into_iter()
            .flatten()
            .map(move |ip| Target::SocketAddr(SocketAddr::new(ip, port)));
        futures::stream::iter(iter).boxed()
    }
}

/// Reverse resolution
impl DnsResolver {
    fn resolve_domain(&self, ip: IpAddr) -> TargetStream {
        let iter = self
            .resolver
            .reverse_lookup(ip)
            .await?
            .into_iter()
            .map(|record| Target::Domain {
                name: record.0.clone(),
                port: None,
            });
        futures::stream::iter(iter).boxed()
    }

    fn resolve_domain_port(&self, sock: SocketAddr) -> TargetStream {
        let iter = self
            .resolver
            .reverse_lookup(sock.ip())
            .await
            .into_iter()
            .flatten()
            .map(move |record| Target::Domain {
                name: record.0.clone(),
                port: Some(sock.port()),
            });
        futures::stream::iter(iter).boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::str::FromStr;

    use futures::StreamExt;
    use rstest::rstest;

    use super::*;
    use crate::Resolve;

    #[rstest]
    #[case("localhost")]
    #[case("google.com")]
    #[case("google.com.")]
    #[tokio::test]
    async fn dns_resolver_works(#[case] input: &str) {
        let target = Target::from_str(input).unwrap();
        let resolver = DnsResolver::system().unwrap();
        let targets: BTreeSet<Target> = resolver.resolve(target).collect().await;
        assert!(targets.len() > 0);
    }
}
