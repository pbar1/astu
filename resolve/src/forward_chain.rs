use anyhow::bail;
use futures::StreamExt;

use crate::CidrResolver;
use crate::DnsResolver;
use crate::FileResolver;
use crate::Resolve;
use crate::ResolveResult;
use crate::Target;

#[derive(Debug, Clone)]
pub struct ForwardChainResolver {
    cidr: CidrResolver,
    file: FileResolver,
    dns: DnsResolver,
}

impl Resolve for ForwardChainResolver {
    fn resolve(&self, target: Target) -> ResolveResult {
        #[allow(unreachable_patterns)]
        match target {
            Target::IpAddr(_) => bounce(target),
            Target::SocketAddr(_) => bounce(target),
            Target::Ssh { .. } => bounce(target),
            Target::Cidr(_) => self.cidr.resolve(target),
            Target::Domain { .. } => self.dns.resolve(target),
            Target::File(_) => self.file.resolve(target),
            #[allow(unreachable_patterns)]
            unsupported => bail!("ForwardChainResolver: unsupported target: {unsupported}"),
        }
    }
}

impl ForwardChainResolver {
    pub fn new() -> Self {
        Self {
            cidr: CidrResolver,
            file: FileResolver,
            dns: DnsResolver,
        }
    }
}

fn bounce(target: Target) -> ResolveResult {
    let stream = futures::stream::iter(vec![Ok(target)]).boxed();
    Ok(stream)
}
