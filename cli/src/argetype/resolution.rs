use astu_resolve::forward_chain;
use astu_resolve::ResolveExt;
use astu_resolve::Target;
use astu_resolve::TargetGraph;
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
    pub async fn resolve(self) -> anyhow::Result<TargetGraph> {
        let chain = forward_chain()?;

        let mut graph = TargetGraph::new();
        for target in self.targets {
            chain.resolve_into_graph(target, &mut graph).await;
        }

        Ok(graph)
    }
}
