use internment::Intern;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::prelude::DiGraphMap;
use petgraph::visit::IntoNodeReferences;

use crate::resolve::Target;

// TODO: Consider using ArenaIntern to release targets on drop
/// Directed graph of unique targets.
#[derive(Debug, Default)]
pub struct TargetGraph {
    graph: DiGraphMap<Intern<Target>, ()>,
}

impl TargetGraph {
    /// Access the inner graph.
    #[must_use]
    pub fn inner(&self) -> &DiGraphMap<Intern<Target>, ()> {
        &self.graph
    }

    /// Access the inner graph mutably.
    pub fn inner_mut(&mut self) -> &mut DiGraphMap<Intern<Target>, ()> {
        &mut self.graph
    }

    /// Add target to the graph with no relation.
    pub fn add_node(&mut self, node: Intern<Target>) {
        self.graph.add_node(node);
    }

    /// Add target relations to the graph. Nodes will be created if they don't
    /// exist yet.
    pub fn add_edge(&mut self, parent: Intern<Target>, child: Intern<Target>) {
        self.graph.add_edge(parent, child, ());
    }

    #[must_use]
    pub fn nodes(&self) -> Vec<Intern<Target>> {
        self.graph.nodes().collect()
    }

    /// Get all of the targets that are leaf nodes (ie, targets that have no
    /// further children).
    #[must_use]
    pub fn leaf_targets(&self) -> Vec<Intern<Target>> {
        self.graph
            .node_references()
            .filter(|(node, _)| {
                self.graph
                    .neighbors_directed(*node, petgraph::Direction::Outgoing)
                    .count()
                    == 0
            })
            .map(|(node, _)| node)
            .collect()
    }

    #[must_use]
    pub fn graphviz(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &|_, node| format!("label=\"{}\"", node.0),
            )
        )
        .replace("digraph {\n", "digraph {\n    rankdir=LR;\n")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_leaf_nodes() {
        let mut g = TargetGraph::default();

        let parent = Target::from_str("10.0.0.0/24").unwrap().intern();
        let child1 = Target::from_str("10.0.0.1").unwrap().intern();
        let child2 = Target::from_str("10.0.0.2").unwrap().intern();

        g.add_edge(parent, child1);
        g.add_edge(parent, child2);

        let got = g.leaf_targets();
        let should = vec![child1, child2];
        assert_eq!(got, should);
    }
}
