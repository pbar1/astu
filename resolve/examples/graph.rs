use std::str::FromStr;

use astu_resolve::DnsResolver;
use astu_resolve::ResolveExt;
use astu_resolve::Target;
use astu_resolve::TargetGraph;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resolver = DnsResolver::try_new()?;
    let target = Target::from_str("salesforce.com").unwrap();
    let mut graph = TargetGraph::new();

    resolver
        .resolve_infallible_into_graph(target, &mut graph)
        .await;

    let dot = graph.graphviz();
    println!("{dot}");

    Ok(())
}
