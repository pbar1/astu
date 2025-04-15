use std::str::FromStr;

use anyhow::Result;
use futures::stream::BoxStream;
use futures::StreamExt;
use ipnet::IpNet;

use crate::resolve::Resolve;
use crate::resolve::Target;

/// Expands CIDR blocks into targets.
#[derive(Debug, Default, Clone, Copy)]
pub struct CidrResolver {
    // FIXME: Use PhantomData to force usage of constructors
}

impl Resolve for CidrResolver {
    fn resolve_fallible(&self, target: Target) -> BoxStream<Result<Target>> {
        match target.cidr() {
            Some(cidr) => self.resolve_cidr(cidr, target),
            _unsupported => futures::stream::empty().boxed(),
        }
    }
}

impl CidrResolver {
    #[allow(clippy::unused_self)]
    fn resolve_cidr(&self, cidr: IpNet, target: Target) -> BoxStream<Result<Target>> {
        let ips = cidr.hosts().map(move |ip| {
            let port = target.port();
            let user = target.user();
            Target::new_ip(ip, port, user)
        });
        futures::stream::iter(ips).boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::resolve::ResolveExt;

    #[rstest]
    #[case("127.0.0.1/32", 1)]
    #[case("127.0.0.0/16", 65534)]
    #[case("::1/128", 1)]
    #[case("::1/112", 65536)]
    #[tokio::test]
    async fn resolve_works(#[case] query: &str, #[case] num: usize) {
        let target = Target::from_str(query).unwrap();
        let resolver = CidrResolver::default();
        let targets = resolver.resolve_set(target).await;
        assert_eq!(targets.len(), num);
    }
}
