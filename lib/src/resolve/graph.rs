use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::IntoNodeReferences;
use petgraph::Direction;
use serde::Deserialize;
use serde::Serialize;

use crate::resolve::Target;

/// Directed graph of unique targets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetGraph {
    graph: DiGraph<Target, ()>,
    map: HashMap<Target, NodeIndex>,
}

impl TargetGraph {
    /// Access the inner graph.
    #[must_use]
    pub fn inner(&self) -> &DiGraph<Target, ()> {
        &self.graph
    }

    /// Access the inner graph mutably.
    pub fn inner_mut(&mut self) -> &mut DiGraph<Target, ()> {
        &mut self.graph
    }

    /// Add target to the graph with no relation.
    pub fn add_node(&mut self, target: &Target) -> NodeIndex {
        if let Some(index) = self.map.get(target) {
            return index.to_owned();
        }
        let index = self.graph.add_node(target.to_owned());
        self.map.insert(target.to_owned(), index);
        index
    }

    /// Add target relations to the graph. Nodes will be created if they don't
    /// exist yet.
    pub fn add_edge(&mut self, parent: &Target, child: &Target) {
        let index_parent = self.add_node(parent);
        let index_child = self.add_node(child);
        self.graph.add_edge(index_parent, index_child, ());
    }

    #[must_use]
    pub fn nodes(&self) -> Vec<Target> {
        self.graph
            .raw_nodes()
            .iter()
            .map(|x| x.weight.clone())
            .collect()
    }

    /// Get all of the targets that are leaf nodes (ie, targets that have no
    /// further children).
    #[must_use]
    pub fn leaf_targets(&self) -> Vec<Target> {
        self.graph
            .node_references()
            .filter(|(node, _)| {
                self.graph
                    .neighbors_directed(*node, Direction::Outgoing)
                    .count()
                    == 0
            })
            .map(|(_node, target)| target.to_owned())
            .collect()
    }

    /// Gets targets grouped into buckets by their parents. If a target has
    /// multiple parents, it will only appear in the first parent's bucket.
    #[must_use]
    pub fn buckets(&self) -> BTreeMap<Target, BTreeSet<Target>> {
        self.graph
            .node_references()
            .fold(BTreeMap::new(), |mut map, (node, target)| {
                if self
                    .graph
                    .neighbors_directed(node, Direction::Outgoing)
                    .count()
                    > 0
                {
                    return map;
                }
                let Some(parent) = self
                    .graph
                    .neighbors_directed(node, Direction::Incoming)
                    .next()
                    .and_then(|node| self.graph.node_weight(node).map(ToOwned::to_owned))
                else {
                    return map;
                };
                map.entry(parent).or_default().insert(target.clone());
                map
            })
    }

    #[must_use]
    pub fn graphviz(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &|_, (_node, target)| format!("label=\"{target}\""),
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

        let a = Target::from_str("10.0.0.0/24").unwrap();
        let b = Target::from_str("10.0.0.1").unwrap();
        let c = Target::from_str("10.0.0.2").unwrap();

        g.add_edge(&a, &b);
        g.add_edge(&a, &c);

        let got = g.leaf_targets();
        let should = vec![b, c];
        assert_eq!(got, should);
    }
}
