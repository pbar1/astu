use std::collections::BTreeSet;

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
        // TODO: Default IP lookup strategy is `Ipv4thenIpv6`. Consider
        // changing it to `Ipv4AndIpv6` to gather all possible IPs.
        let targets = self
            .resolver
            .lookup_ip(query)
            .await?
            .iter()
            .map(|ip| match ip {
                std::net::IpAddr::V4(x) => Target::Ipv4Addr(x),
                std::net::IpAddr::V6(x) => Target::Ipv6Addr(x),
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
