mod dns;
mod ip;
mod target;

use std::collections::BTreeSet;

pub use dns::DnsResolver;
pub use ip::IpResolver;
pub use target::Target;

#[async_trait::async_trait]
pub trait Resolve {
    async fn resolve(&self, query: &str) -> anyhow::Result<BTreeSet<Target>>;
}
