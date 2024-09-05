use anyhow::bail;

use crate::DnsResolver;
use crate::Resolve;
use crate::ResolveResult;
use crate::Target;

#[derive(Debug, Clone)]
pub struct ReverseChainResolver {
    dns: DnsResolver,
}

impl Resolve for ReverseChainResolver {
    fn resolve(&self, target: Target) -> ResolveResult {
        match target {
            Target::IpAddr(_) => self.dns.resolve(target),
            Target::SocketAddr(_) => self.dns.resolve(target),
            unsupported => bail!("ReverseChainResolver: unsupported target: {unsupported}"),
        }
    }
}

impl ReverseChainResolver {
    pub fn new() -> Self {
        Self { dns: DnsResolver }
    }
}
