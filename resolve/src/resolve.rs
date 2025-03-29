use std::collections::BTreeSet;

use anyhow::Result;
use futures::stream::BoxStream;
use futures::StreamExt;

use crate::Target;
use crate::TargetGraph;

/// Map targets to targets.
///
/// Unsupported target types should just return empty streams instead of
/// erroring.
pub trait Resolve {
    /// Resolve a target query.
    fn resolve_fallible(&self, target: Target) -> BoxStream<Result<Target>>;
}

/// An extension trait for [`Resolve`] that provides a variety of convenient
/// combinator functions.
pub trait ResolveExt: Resolve {
    /// Like [`Resolve::resolve`], but elides all errors.
    fn resolve(&self, target: Target) -> BoxStream<Target>;

    /// Collects all targets into a new set.
    async fn resolve_set(&self, target: Target) -> BTreeSet<Target>;

    /// Collects all targets into a vec.
    async fn resolve_into_vec(&self, target: Target, vec: &mut Vec<Target>);

    /// Collects all targets into an existing graph.
    async fn resolve_into_graph(&self, target: Target, graph: &mut TargetGraph);
}

impl<R> ResolveExt for R
where
    R: Resolve,
{
    fn resolve(&self, target: Target) -> BoxStream<Target> {
        self.resolve_fallible(target)
            .map(futures::stream::iter)
            .flatten()
            .boxed()
    }

    async fn resolve_set(&self, target: Target) -> BTreeSet<Target> {
        self.resolve(target).collect().await
    }

    async fn resolve_into_vec(&self, target: Target, vec: &mut Vec<Target>) {
        let mut targets = self.resolve(target);
        while let Some(target) = targets.next().await {
            vec.push(target);
        }
    }

    async fn resolve_into_graph(&self, target: Target, graph: &mut TargetGraph) {
        let parent = target.clone().intern();
        graph.add_node(parent);

        let mut targets = self.resolve(target);
        while let Some(target) = targets.next().await {
            graph.add_edge(parent, target.intern());
        }
    }
}
