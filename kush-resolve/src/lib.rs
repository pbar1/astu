mod dns;
mod ip;
mod target;

pub use dns::DnsResolver;
use futures::Stream;
pub use ip::IpResolver;
pub use target::Target;

pub trait Resolve {
    fn resolve(&self, target: Target) -> impl Stream<Item = Target>;
}

pub struct ResolveChain {
    resolvers: Vec<Box<dyn Resolve + Send + Sync>>,
}

impl ResolveChain {
    pub fn forward() -> anyhow::Result<Self> {
        let mut resolvers: Vec<Box<dyn Resolve + Send + Sync>> = Vec::new();
        resolvers.push(Box::new(IpResolver));
        resolvers.push(Box::new(DnsResolver::system()?));
        Ok(Self { resolvers })
    }
}
