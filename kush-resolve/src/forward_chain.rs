use anyhow::bail;
use futures::StreamExt;

use crate::CidrResolver;
use crate::FileResolver;
use crate::Resolve;
use crate::ResolveResult;
use crate::Target;

pub struct ForwardChainResolver {
    cidr: CidrResolver,
    file: FileResolver,
}

impl Resolve for ForwardChainResolver {
    fn resolve(&self, target: Target) -> ResolveResult {
        match target {
            Target::File(_) => self.file.resolve(target),
            rest => self.resolve_inner(rest),
        }
    }
}

impl ForwardChainResolver {
    pub fn new() -> Self {
        Self {
            cidr: CidrResolver,
            file: FileResolver,
        }
    }

    fn resolve_inner(&self, target: Target) -> ResolveResult {
        match target {
            Target::IpAddr(_) => bounce(target),
            Target::SocketAddr(_) => bounce(target),
            Target::Ssh { .. } => bounce(target),
            Target::Cidr(_) => self.cidr.resolve(target),
            unsupported => bail!("ForwardChainResolver: unsupported target: {unsupported}"),
        }
    }
}

fn bounce(target: Target) -> ResolveResult {
    let stream = futures::stream::iter(vec![Ok(target)]).boxed();
    Ok(stream)
}
