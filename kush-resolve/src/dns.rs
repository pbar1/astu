use std::collections::BTreeSet;
use std::str::FromStr;

use anyhow::bail;
use hickory_resolver::Name;
use hickory_resolver::TokioAsyncResolver;

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

#[async_trait::async_trait]
impl super::Resolve for DnsResolver {
    async fn resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>> {
        let target = Target::from_str(&query)?;

        let targets = match target {
            Target::Domain(name) => self.resolve_ip(name).await?,
            Target::DomainPort { name, port } => self.resolve_sock(name, port).await?,
            unsupported => bail!("unsupported target for IpResolver: {unsupported}"),
        };

        Ok(targets)
    }
}

// TODO: Default IP lookup strategy is `Ipv4thenIpv6`. Consider
// changing it to `Ipv4AndIpv6` to gather all possible IPs.
impl DnsResolver {
    async fn resolve_ip(&self, name: Name) -> anyhow::Result<BTreeSet<Target>> {
        let targets = self
            .resolver
            .lookup_ip(name)
            .await?
            .iter()
            .map(|ip| match ip {
                std::net::IpAddr::V4(x) => Target::Ipv4Addr(x),
                std::net::IpAddr::V6(x) => Target::Ipv6Addr(x),
            })
            .collect();
        Ok(targets)
    }

    async fn resolve_sock(&self, name: Name, port: u16) -> anyhow::Result<BTreeSet<Target>> {
        let targets = self
            .resolver
            .lookup_ip(name)
            .await?
            .iter()
            .map(|ip| match ip {
                std::net::IpAddr::V4(x) => {
                    Target::SocketAddrV4(std::net::SocketAddrV4::new(x, port))
                }
                std::net::IpAddr::V6(x) => {
                    Target::SocketAddrV6(std::net::SocketAddrV6::new(x, port, 0, 0))
                }
            })
            .collect();
        Ok(targets)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::Resolve;

    #[rstest]
    #[case("localhost")]
    #[case("google.com")]
    #[case("google.com.")]
    #[tokio::test]
    async fn dns_resolver_works(#[case] search_term: &str) {
        let resolver = DnsResolver::system().unwrap();
        let targets = resolver.resolve(search_term).await.unwrap();
        dbg!(targets.clone());
        assert!(targets.len() > 0);
    }
}
