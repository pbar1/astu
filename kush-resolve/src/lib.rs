// mod dns;
// mod file;
mod cidr;
mod target;
// mod uri;

use std::pin::Pin;

// pub use dns::DnsResolver;
// pub use file::FileResolver;
use futures::Stream;
use futures::StreamExt;
// pub use ip::IpResolver;
pub use target::Target;
// pub use uri::UriResolver;

pub type TargetResult = anyhow::Result<Target>;
pub type TargetResultStream = Pin<Box<dyn Stream<Item = TargetResult> + Send>>;
pub type TargetStream = Pin<Box<dyn Stream<Item = Target>>>;

pub trait Resolve {
    /// Expands a [`Target`] into a stream of [`TargetResult`].
    ///
    /// Use the functions in [`ResolveExt`] for friendlier stream ergonomics.
    fn resolve(&self, target: Target) -> anyhow::Result<TargetResultStream>;
}

pub trait ResolveExt: Resolve + Send + Sync {
    /// Returns a stream of [`Target`] while ignoring all errors.
    fn resolve_infallible(&self, target: Target) -> TargetStream;
}

impl<T> ResolveExt for T
where
    T: Resolve + Send + Sync,
{
    /// Returns a stream of [`Target`] while ignoring all errors.
    fn resolve_infallible(&self, target: Target) -> TargetStream {
        let resolved = self.resolve(target);

        // TODO: Log errors using `inspect` before flattening
        futures::stream::iter(resolved)
            .flatten()
            .map(futures::stream::iter)
            .flatten()
            .boxed()
    }
}
