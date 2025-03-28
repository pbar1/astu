use std::str::FromStr;

use astu_resolve::Target;
use astu_resolve::TargetGraph;

fn main() -> anyhow::Result<()> {
    let mut g = TargetGraph::new();

    let parent = Target::from_str("10.0.0.0/24")?.intern();
    let child1 = Target::from_str("10.0.0.1")?.intern();
    let child2 = Target::from_str("10.0.0.2")?.intern();

    g.add_edge(parent, child1);
    g.add_edge(parent, child2);

    let dot = g.graphviz();
    println!("{dot}");

    Ok(())
}
