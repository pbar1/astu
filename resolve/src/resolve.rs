use std::collections::BTreeSet;

use anyhow::Result;
use futures::stream::BoxStream;
use futures::StreamExt;

use crate::Target;

/// Given initial targets as queries, [`Resolve`] expands them into more
/// targets if possible.
///
/// Unsupported target types should just return empty streams instead of
/// erroring.
pub trait Resolve {
    /// Resolve a target query.
    fn resolve(&self, target: Target) -> BoxStream<Result<Target>>;
}

/// An extension trait for [`Resolve`] that provides a variety of convenient
/// combinator functions.
pub trait ResolveExt: Resolve {
    /// Like [`Resolve::resolve`], but elides all errors.
    fn resolve_infallible(&self, target: Target) -> BoxStream<Target>;

    /// Collects all targets into a unique set.
    async fn resolve_infallible_set(&self, target: Target) -> BTreeSet<Target>;
}

impl<R> ResolveExt for R
where
    R: Resolve,
{
    fn resolve_infallible(&self, target: Target) -> BoxStream<Target> {
        self.resolve(target)
            .map(futures::stream::iter)
            .flatten()
            .boxed()
    }

    async fn resolve_infallible_set(&self, target: Target) -> BTreeSet<Target> {
        self.resolve_infallible(target).collect().await
    }
}
