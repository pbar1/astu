use std::net::IpAddr;
use std::net::SocketAddr;

use async_stream::stream;
use futures::Stream;
use hickory_resolver::Name;
use hickory_resolver::TokioAsyncResolver;

use crate::Resolve;
use crate::Target;

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl DnsResolver {
    pub fn system() -> anyhow::Result<Self> {
        let (config, options) = hickory_resolver::system_conf::read_system_conf()?;
        let resolver = TokioAsyncResolver::tokio(config, options);
        Ok(Self { resolver })
    }
}

impl Resolve for DnsResolver {
    fn resolve(&self, target: Target) -> impl Stream<Item = Target> {
        stream! {
            match target {
                // Forward
                Target::Domain(name) => {
                    for target in self.resolve_ip(name).await {
                        yield target
                    }
                },
                Target::DomainPort{ name, port } => {
                    for target in self.resolve_sock(name, port).await {
                        yield target
                    }
                },

                // Reverse
                Target::Ipv4Addr(ip) => {
                    for target in self.resolve_domain(ip.into()).await {
                        yield target
                    }
                },
                Target::Ipv6Addr(ip) => {
                    for target in self.resolve_domain(ip.into()).await {
                        yield target
                    }
                },
                Target::SocketAddrV4(sock) => {
                    for target in self.resolve_domain_port(sock.into()).await {
                        yield target
                    }
                },
                Target::SocketAddrV6(sock) => {
                    for target in self.resolve_domain_port(sock.into()).await {
                        yield target
                    }
                },

                _unsupported => return,
            };
        }
    }
}

// Forward resolution
impl DnsResolver {
    // TODO: Default IP lookup strategy is `Ipv4thenIpv6`. Consider
    // changing it to `Ipv4AndIpv6` to gather all possible IPs.

    async fn resolve_ip(&self, name: Name) -> impl Iterator<Item = Target> {
        self.resolver
            .lookup_ip(name)
            .await
            .into_iter()
            .flatten()
            .map(|ip| match ip {
                std::net::IpAddr::V4(x) => Target::Ipv4Addr(x),
                std::net::IpAddr::V6(x) => Target::Ipv6Addr(x),
            })
    }

    async fn resolve_sock(&self, name: Name, port: u16) -> impl Iterator<Item = Target> {
        self.resolver
            .lookup_ip(name)
            .await
            .into_iter()
            .flatten()
            .map(move |ip| match ip {
                std::net::IpAddr::V4(x) => {
                    Target::SocketAddrV4(std::net::SocketAddrV4::new(x, port))
                }
                std::net::IpAddr::V6(x) => {
                    Target::SocketAddrV6(std::net::SocketAddrV6::new(x, port, 0, 0))
                }
            })
    }
}

/// Reverse resolution
impl DnsResolver {
    async fn resolve_domain(&self, ip: IpAddr) -> impl Iterator<Item = Target> {
        self.resolver
            .reverse_lookup(ip)
            .await
            .into_iter()
            .flatten()
            .map(|record| Target::Domain(record.0.clone()))
    }

    async fn resolve_domain_port(&self, sock: SocketAddr) -> impl Iterator<Item = Target> {
        self.resolver
            .reverse_lookup(sock.ip())
            .await
            .into_iter()
            .flatten()
            .map(move |record| Target::DomainPort {
                name: record.0.clone(),
                port: sock.port(),
            })
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
