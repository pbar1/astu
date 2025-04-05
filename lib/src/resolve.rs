#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(async_fn_in_trait)]

mod graph;
mod provider;
mod resolve;
mod target;

pub use graph::TargetGraph;
pub use provider::chain::ChainResolver;
pub use provider::cidr::CidrResolver;
pub use provider::dns::DnsResolver;
pub use provider::file::FileResolver;
pub use resolve::Resolve;
pub use resolve::ResolveExt;
pub use target::Target;

/// Create the default chain of forward resolvers.
pub fn forward_chain() -> anyhow::Result<ChainResolver> {
    let chain = ChainResolver::new()
        .with(FileResolver::new())
        .with(CidrResolver::new())
        .with(DnsResolver::try_new()?);
    Ok(chain)
}

/// Create the default chain of reverse resolvers.
pub fn reverse_chain() -> anyhow::Result<ChainResolver> {
    let chain = ChainResolver::new().with(
        DnsResolver::try_new()?
            .with_forward(false)
            .with_reverse(true),
    );
    Ok(chain)
}
