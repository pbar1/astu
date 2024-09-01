// mod dns;
mod target;

use std::collections::BTreeSet;

pub use target::Target;

#[async_trait::async_trait]
pub trait Resolve {
    async fn resolve(&self) -> anyhow::Result<BTreeSet<Target>>;
}
