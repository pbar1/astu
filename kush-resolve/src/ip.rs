use std::collections::BTreeSet;
use std::str::FromStr;

use async_stream::stream;
use futures::Stream;
use futures::StreamExt;

use crate::Resolve2;
use crate::Target;

pub struct IpResolver;

#[async_trait::async_trait]
impl super::Resolve for IpResolver {
    async fn resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>> {
        let target = Target::from_str(query)?;
        let targets = self.resolve2(target).collect().await;
        Ok(targets)
    }
}

impl super::Resolve2 for IpResolver {
    fn resolve2(&self, target: Target) -> impl Stream<Item = Target> {
        stream! {
            match target {
                Target::Ipv4Addr(_) => yield target,
                Target::Ipv6Addr(_) => yield target,
                Target::SocketAddrV4(_) => yield target,
                Target::SocketAddrV6(_) => yield target,
                Target::Ipv4Net(x) => {
                    for host in x.hosts() {
                        yield Target::Ipv4Addr(host);
                    }
                }
                Target::Ipv6Net(x) => {
                    for host in x.hosts() {
                        yield Target::Ipv6Addr(host);
                    }
                }
                _rest => return,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::pin_mut;
    use futures::StreamExt;
    use rstest::rstest;

    use super::*;
    use crate::Resolve2;

    #[rstest]
    #[case("127.0.0.1", 1)]
    #[case("::1", 1)]
    #[case("127.0.0.1:22", 1)]
    #[case("[::1]:22", 1)]
    #[case("10.0.0.0/32", 1)]
    #[case("10.0.0.0/16", 65534)]
    #[case("::/128", 1)]
    #[case("::/112", 65536)]
    #[tokio::test]
    async fn resolve2_works(#[case] query: &str, #[case] num: usize) {
        let target = Target::from_str(query).unwrap();
        let resolver = IpResolver;
        let targets: BTreeSet<Target> = resolver.resolve2(target).collect().await;
        assert_eq!(targets.len(), num);
    }

    #[rstest]
    #[case("0.0.0.0/0")]
    #[case("::/0")]
    #[tokio::test]
    async fn resolve2_huge(#[case] query: &str) {
        let target = Target::from_str(query).unwrap();
        let resolver = IpResolver;
        let resolved = resolver.resolve2(target);
        pin_mut!(resolved);
        let mut counter = 0u128;
        while let Some(_) = resolved.next().await {
            counter += 1;
            if counter >= 100_000_000 {
                break;
            }
        }
        assert!(counter >= 1_000_000);
    }
}
