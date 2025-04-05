mod graph;
pub mod provider;
mod target;

use std::collections::BTreeSet;

use anyhow::Result;
use futures::stream::BoxStream;
use futures::StreamExt;

pub use self::graph::TargetGraph;
pub use self::target::Target;

/// Map targets to targets.
///
/// Unsupported target types should just return empty streams instead of
/// erroring.
pub trait Resolve {
    /// Resolve a target query.
    fn resolve_fallible(&self, target: Target) -> BoxStream<Result<Target>>;

    /// Like [`Resolve::resolve_fallible`], but ignores all errors.
    fn resolve(&self, target: Target) -> BoxStream<Target> {
        self.resolve_fallible(target)
            .map(futures::stream::iter)
            .flatten()
            .boxed()
    }

    /// Resolve multiple target queries at once.
    ///
    /// The default implementation simply resolves in serial. Override this if a
    /// more efficient implementation exists.
    fn bulk_resolve_fallible(&self, targets: Vec<Target>) -> BoxStream<Result<Target>>
    where
        Self: Sync,
    {
        futures::stream::iter(targets)
            .map(|t| self.resolve_fallible(t))
            .flatten()
            .boxed()
    }

    /// Like [`Resolve::bulk_resolve_fallible`], but ignores all errors.
    fn bulk_resolve(&self, targets: Vec<Target>) -> BoxStream<Target>
    where
        Self: Sync,
    {
        self.bulk_resolve_fallible(targets)
            .map(futures::stream::iter)
            .flatten()
            .boxed()
    }
}

/// An extension trait for [`Resolve`] that provides a variety of convenient
/// combinator functions.
///
/// This is especially useful for holding the `async` functions would otherwise
/// make the main trait dyn-incompatible.
pub trait ResolveExt: Resolve {
    /// Resolve targets to a new set.
    async fn resolve_set(&self, target: Target) -> BTreeSet<Target>;

    /// Resolve targets into an existing set.
    async fn resolve_into_set(&self, target: Target, set: &mut BTreeSet<Target>);

    /// Resolve targets into an existing graph.
    async fn resolve_into_graph(&self, target: Target, graph: &mut TargetGraph);

    /// Resolve targets into an existing graph, with the parent->child
    /// relationship reversed.
    ///
    /// This is useful when called by reverse resolvers.
    async fn resolve_into_graph_reverse(&self, target: Target, graph: &mut TargetGraph);

    /// Like [`ResolveExt::resolve_set`] but for bulk targets.
    async fn bulk_resolve_set(&self, targets: Vec<Target>) -> BTreeSet<Target>
    where
        Self: Sync;

    /// Like [`ResolveExt::resolve_into_set`] but for bulk targets.
    async fn bulk_resolve_into_set(&self, targets: Vec<Target>, set: &mut BTreeSet<Target>)
    where
        Self: Sync;
}

impl<R> ResolveExt for R
where
    R: Resolve,
{
    async fn resolve_set(&self, target: Target) -> BTreeSet<Target> {
        self.resolve(target).collect().await
    }

    async fn resolve_into_set(&self, target: Target, set: &mut BTreeSet<Target>) {
        let mut targets = self.resolve(target);
        while let Some(target) = targets.next().await {
            set.insert(target);
        }
    }

    async fn resolve_into_graph(&self, target: Target, graph: &mut TargetGraph) {
        let parent = target.clone().intern();
        graph.add_node(parent);

        let mut targets = self.resolve(target);
        while let Some(child) = targets.next().await {
            // Avoid having targets refer to themselves
            if child != *parent {
                graph.add_edge(parent, child.intern());
            }
        }
    }

    async fn resolve_into_graph_reverse(&self, target: Target, graph: &mut TargetGraph) {
        let child = target.clone().intern();
        graph.add_node(child);

        let mut targets = self.resolve(target);
        while let Some(parent) = targets.next().await {
            // Avoid having targets refer to themselves
            if parent != *child {
                graph.add_edge(parent.intern(), child);
            }
        }
    }

    async fn bulk_resolve_set(&self, targets: Vec<Target>) -> BTreeSet<Target>
    where
        Self: Sync,
    {
        self.bulk_resolve(targets).collect().await
    }

    async fn bulk_resolve_into_set(&self, targets: Vec<Target>, set: &mut BTreeSet<Target>)
    where
        Self: Sync,
    {
        let mut targets = self.bulk_resolve(targets);
        while let Some(target) = targets.next().await {
            set.insert(target);
        }
    }
}
