use anyhow::bail;
use futures::StreamExt;

use crate::Resolve;
use crate::ResolveResult;
use crate::Target;

#[derive(Debug, Clone)]
pub struct CidrResolver;

impl Resolve for CidrResolver {
    fn resolve(&self, target: Target) -> ResolveResult {
        let cidr = match target {
            Target::Cidr(cidr) => cidr,
            unsupported => bail!("CidrResolver: unsupported target: {unsupported}"),
        };
        let iter = cidr.hosts().map(|ip| Ok(Target::from(ip)));
        let stream = futures::stream::iter(iter).boxed();
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::ResolveExt;

    #[rstest]
    #[case("127.0.0.1/32", 1)]
    #[case("127.0.0.0/16", 65534)]
    #[case("::1/128", 1)]
    #[case("::1/112", 65536)]
    #[tokio::test]
    async fn resolve_works(#[case] query: &str, #[case] num: usize) {
        let target = Target::from_str(query).unwrap();
        let resolver = CidrResolver;
        let targets: BTreeSet<Target> = resolver.resolve_infallible(target).collect().await;
        assert_eq!(targets.len(), num);
    }
}
