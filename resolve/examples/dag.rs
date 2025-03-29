use std::str::FromStr;

use astu_resolve::DnsResolver;
use astu_resolve::ResolveExt;
use astu_resolve::Target;
use astu_resolve::TargetGraph;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let parent = Target::from_str("salesforce.com").unwrap();
    let resolver = DnsResolver;

    let targets = resolver
        .resolve_infallible(parent.clone())
        .collect::<Vec<_>>()
        .await;

    let parent = parent.intern();
    let other = Target::from_str("10.0.0.0/24")?.intern();

    let mut g = TargetGraph::new();

    let mut i = 0;
    for target in targets {
        let child = target.intern();

        g.add_edge(parent, child);

        if i % 2 == 0 {
            g.add_edge(other, child);
        }

        i += 1;
    }

    let dot = g.graphviz();
    println!("{dot}");

    Ok(())
}
