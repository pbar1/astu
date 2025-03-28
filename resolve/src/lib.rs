#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod cidr;
mod dns;
mod file;
mod forward_chain;
mod graph;
mod reverse_chain;
mod target;

use std::pin::Pin;

use futures::Stream;
use futures::StreamExt;

pub use crate::cidr::CidrResolver;
pub use crate::dns::DnsResolver;
pub use crate::file::FileResolver;
pub use crate::forward_chain::ForwardChainResolver;
pub use crate::graph::TargetGraph;
pub use crate::reverse_chain::ReverseChainResolver;
pub use crate::target::Target;

pub type TargetResult = anyhow::Result<Target>;
pub type ResolveResult = anyhow::Result<TargetResultStream>;
pub type TargetResultStream = Pin<Box<dyn Stream<Item = TargetResult> + Send>>;
pub type TargetStream = Pin<Box<dyn Stream<Item = Target> + Send>>;

pub trait Resolve {
    /// Expands a [`Target`] into a stream of [`TargetResult`].
    ///
    /// Use the functions in [`ResolveExt`] for friendlier stream ergonomics.
    fn resolve(&self, target: Target) -> ResolveResult;
}

pub trait ResolveExt: Resolve {
    /// Expands a [`Target`] into a stream of [`Target`] while dropping all
    /// errors.
    fn resolve_infallible(&self, target: Target) -> TargetStream;
}

impl<T> ResolveExt for T
where
    T: Resolve,
{
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
