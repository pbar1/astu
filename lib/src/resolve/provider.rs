mod chain;
mod cidr;
mod dns;
mod file;
mod k8s;

pub use self::chain::ChainResolver;
pub use self::cidr::CidrResolver;
pub use self::dns::DnsResolver;
pub use self::file::FileResolver;

/// Create the default chain of forward resolvers.
///
/// # Errors
///
/// If any of the resolvers in the chain fail to build.
pub fn forward_chain() -> anyhow::Result<ChainResolver> {
    let chain = ChainResolver::default()
        .with(FileResolver::default())
        .with(CidrResolver::default())
        .with(DnsResolver::try_new()?);
    Ok(chain)
}

/// Create the default chain of reverse resolvers.
///
/// # Errors
///
/// If any of the resolvers in the chain fail to build.
pub fn reverse_chain() -> anyhow::Result<ChainResolver> {
    let chain = ChainResolver::default().with(
        DnsResolver::try_new()?
            .with_forward(false)
            .with_reverse(true),
    );
    Ok(chain)
}
