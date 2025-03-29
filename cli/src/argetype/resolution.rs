use astu_resolve::ChainResolver;
use astu_resolve::CidrResolver;
use astu_resolve::DnsResolver;
use astu_resolve::FileResolver;
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
    pub async fn resolve(self) -> TargetGraph {
        let chain = ChainResolver::new()
            .with(FileResolver::new())
            .with(CidrResolver::new())
            .with(DnsResolver::try_new().unwrap());

        let mut graph = TargetGraph::new();
        for target in self.targets {
            chain.resolve_into_graph(target, &mut graph).await;
        }

        graph
    }
}
