use std::collections::BTreeSet;
use std::str::FromStr;

use anyhow::bail;

use crate::Target;

pub struct IpResolver;

#[async_trait::async_trait]
impl super::Resolve for IpResolver {
    async fn resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>> {
        let target = Target::from_str(query)?;

        let targets = match &target {
            Target::Ipv4Addr(_) => BTreeSet::from([target]),
            Target::Ipv6Addr(_) => BTreeSet::from([target]),
            Target::SocketAddrV4(_) => BTreeSet::from([target]),
            Target::SocketAddrV6(_) => BTreeSet::from([target]),
            Target::Ipv4Net(x) => x.hosts().map(Target::Ipv4Addr).collect(),
            Target::Ipv6Net(x) => x.hosts().map(Target::Ipv6Addr).collect(),
            unsupported => bail!("unsupported target for IpResolver: {unsupported}"),
        };

        Ok(targets)
    }
}
