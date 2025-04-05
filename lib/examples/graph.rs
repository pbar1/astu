use std::str::FromStr;

use astu::resolve::provider::DnsResolver;
use astu::resolve::ResolveExt;
use astu::resolve::Target;
use astu::resolve::TargetGraph;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resolver = DnsResolver::try_new()?;
    let target = Target::from_str("salesforce.com").unwrap();
    let mut graph = TargetGraph::default();

    resolver.resolve_into_graph(target, &mut graph).await;

    let dot = graph.graphviz();
    println!("{dot}");

    Ok(())
}
