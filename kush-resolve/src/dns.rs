use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::bail;
use futures::FutureExt;
use futures::StreamExt;
use futures::TryStreamExt;
use hickory_resolver::Name;
use hickory_resolver::TokioAsyncResolver;

use crate::Resolve;
use crate::ResolveResult;
use crate::Target;
use crate::TargetStream;

#[derive(Debug, Clone)]
pub struct DnsResolver;

impl Resolve for DnsResolver {
    fn resolve(&self, target: Target) -> ResolveResult {
        match target {
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
        }
    }
}

impl DnsResolver {}

// Forward resolution
impl DnsResolver {
    fn resolve_ip(&self, name: Name) -> ResolveResult {
        let stream = lookup_ips(name)
            .into_stream()
            .map(futures::stream::iter)
            .flatten()
            .map(|ip| Ok(Target::from(ip)))
            .boxed();
        Ok(stream)
    }

    fn resolve_sock(&self, name: Name, port: u16) -> ResolveResult {
        let stream = lookup_ips(name)
            .into_stream()
            .map(futures::stream::iter)
            .flatten()
            .map(move |ip| Ok(Target::from(SocketAddr::new(ip, port))))
            .boxed();
        Ok(stream)
    }
}

/// Reverse resolution
impl DnsResolver {
    fn resolve_domain(&self, ip: IpAddr) -> ResolveResult {
        let stream = lookup_domains(ip)
            .into_stream()
            .map(futures::stream::iter)
            .flatten()
            .map(|name| Ok(Target::from(name)))
            .boxed();
        Ok(stream)
    }

    fn resolve_domain_port(&self, sock: SocketAddr) -> ResolveResult {
        let stream = lookup_domains(sock.ip())
            .into_stream()
            .map(futures::stream::iter)
            .flatten()
            .map(move |name| {
                Ok(Target::Domain {
                    name,
                    port: Some(sock.port()),
                })
            })
            .boxed();
        Ok(stream)
    }
}

fn get_dns_client() -> anyhow::Result<TokioAsyncResolver> {
    let (config, options) = hickory_resolver::system_conf::read_system_conf()?;
    let dns = TokioAsyncResolver::tokio(config, options);
    Ok(dns)
}

// TODO: Default IP lookup strategy is `Ipv4thenIpv6`. Consider changing it to
// `Ipv4AndIpv6` to gather all possible IPs.
async fn lookup_ips(name: Name) -> Vec<IpAddr> {
    let Ok(dns) = get_dns_client() else {
        return Vec::new();
    };
    let Ok(lookup) = dns.lookup_ip(name).await else {
        return Vec::new();
    };
    lookup.into_iter().collect()
}

async fn lookup_domains(ip: IpAddr) -> Vec<Name> {
    let Ok(dns) = get_dns_client() else {
        return Vec::new();
    };
    let Ok(lookup) = dns.reverse_lookup(ip).await else {
        return Vec::new();
    };
    lookup.into_iter().map(|x| x.0).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::str::FromStr;

    use futures::StreamExt;
    use rstest::rstest;

    use super::*;
    use crate::ResolveExt;

    #[rstest]
    #[case("localhost")]
    #[case("google.com")]
    #[case("google.com.")]
    #[tokio::test]
    async fn dns_resolver_works(#[case] input: &str) {
        let target = Target::from_str(input).unwrap();
        let resolver = DnsResolver;
        let targets: BTreeSet<Target> = resolver.resolve_infallible(target).collect().await;
        assert!(targets.len() > 0);
    }
}
