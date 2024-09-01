mod dns;
mod ip;
mod target;

use std::collections::BTreeSet;

use anyhow::bail;
pub use dns::DnsResolver;
pub use ip::IpResolver;
pub use target::Target;

#[async_trait::async_trait]
pub trait Resolve {
    async fn resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>>;
}

pub struct ResolveChain {
    resolvers: Vec<Box<dyn Resolve + Send + Sync>>,
}

impl ResolveChain {
    pub fn try_default() -> anyhow::Result<Self> {
        let mut resolvers: Vec<Box<dyn Resolve + Send + Sync>> = Vec::new();
        resolvers.push(Box::new(IpResolver));
        resolvers.push(Box::new(DnsResolver::system()?));
        Ok(Self { resolvers })
    }
}

#[async_trait::async_trait]
impl Resolve for ResolveChain {
    async fn resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>> {
        for resolver in &self.resolvers {
            if let Ok(t) = resolver.resolve(query).await {
                return Ok(t);
            };
        }
        bail!("no targets found for query: {query}");
    }
}

#[async_trait::async_trait]
pub trait ReverseResolve {
    async fn reverse_resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>>;
}

pub struct ReverseResolveChain {
    resolvers: Vec<Box<dyn ReverseResolve + Send + Sync>>,
}

impl ReverseResolveChain {
    pub fn try_default() -> anyhow::Result<Self> {
        let mut resolvers: Vec<Box<dyn ReverseResolve + Send + Sync>> = Vec::new();
        resolvers.push(Box::new(DnsResolver::system()?));
        Ok(Self { resolvers })
    }
}

#[async_trait::async_trait]
impl ReverseResolve for ReverseResolveChain {
    async fn reverse_resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>> {
        for resolver in &self.resolvers {
            if let Ok(t) = resolver.reverse_resolve(query).await {
                return Ok(t);
            };
        }
        bail!("no targets found for query: {query}");
    }
}
