use std::collections::BTreeSet;

use astu::resolve::forward_chain;
use astu::resolve::reverse_chain;
use astu::resolve::ResolveExt;
use astu::resolve::Target;
use astu::resolve::TargetGraph;
use clap::Args;

const HEADING: Option<&str> = Some("Target Resolution Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ResolutionArgs {
    /// Target query. Pass `-` to read from stdin.
    #[clap(short = 'T', long = "targets", help_heading = HEADING)]
    pub targets: Vec<Target>,
}

impl ResolutionArgs {
    pub async fn set(&self) -> anyhow::Result<BTreeSet<Target>> {
        let chain = forward_chain()?;
        let set = chain.bulk_resolve_set(self.targets.clone()).await;
        Ok(set)
    }

    pub async fn _graph(self) -> anyhow::Result<TargetGraph> {
        let chain = forward_chain()?;

        let mut graph = TargetGraph::new();
        for target in self.targets {
            chain.resolve_into_graph(target, &mut graph).await;
        }

        Ok(graph)
    }

    pub async fn graph_full(self) -> anyhow::Result<TargetGraph> {
        let fwd = forward_chain()?;
        let rev = reverse_chain()?;

        let mut graph = TargetGraph::new();
        for target in self.targets {
            fwd.resolve_into_graph(target, &mut graph).await;
        }
        for target in graph.nodes() {
            let target = (*target).clone();
            rev.resolve_into_graph_reverse(target, &mut graph).await;
        }

        Ok(graph)
    }
}
