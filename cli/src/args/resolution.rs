use std::collections::BTreeSet;
use std::str::FromStr;

use astu::resolve::provider::forward_chain;
use astu::resolve::provider::reverse_chain;
use astu::resolve::ResolveExt;
use astu::resolve::Target;
use astu::resolve::TargetGraph;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use clap::Args;

const HEADING: Option<&str> = Some("Target Resolution Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ResolutionArgs {
    /// Target query.
    #[clap(short = 'T', long = "target", help_heading = HEADING)]
    pub targets: Vec<Target>,

    /// Path to file with target URIs.
    #[clap(short = 'f', long = "target-file", help_heading = HEADING)]
    pub target_files: Vec<String>,
}

impl ResolutionArgs {
    pub async fn seed_targets(&self, stdin_targets: Option<&str>) -> anyhow::Result<Vec<Target>> {
        let mut targets = self.targets.clone();
        for file in &self.target_files {
            if file == "-" || file == "/dev/stdin" {
                if let Some(stdin) = stdin_targets {
                    for line in stdin.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        targets.push(Target::from_str(line)?);
                    }
                } else {
                    let stdin = tokio::io::stdin();
                    let mut lines = BufReader::new(stdin).lines();
                    while let Some(line) = lines.next_line().await? {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        targets.push(Target::from_str(line)?);
                    }
                }
            } else {
                let file = tokio::fs::File::open(file).await?;
                let mut lines = BufReader::new(file).lines();
                while let Some(line) = lines.next_line().await? {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    targets.push(Target::from_str(line)?);
                }
            }
        }
        Ok(targets)
    }

    pub async fn set(&self) -> anyhow::Result<BTreeSet<Target>> {
        self.set_with_stdin(None).await
    }

    pub async fn set_with_stdin(&self, stdin_targets: Option<&str>) -> anyhow::Result<BTreeSet<Target>> {
        let chain = forward_chain()?;
        let set = chain.bulk_resolve_set(self.seed_targets(stdin_targets).await?).await;
        Ok(set)
    }

    pub async fn _graph(self) -> anyhow::Result<TargetGraph> {
        let chain = forward_chain()?;

        let mut graph = TargetGraph::default();
        for target in self.targets {
            chain.resolve_into_graph(target, &mut graph).await;
        }

        Ok(graph)
    }

    pub async fn graph_full(self) -> anyhow::Result<TargetGraph> {
        let fwd = forward_chain()?;
        let rev = reverse_chain()?;

        let mut graph = TargetGraph::default();
        for target in self.targets {
            fwd.resolve_into_graph(target, &mut graph).await;
        }
        for target in graph.nodes() {
            let target = target.clone();
            rev.resolve_into_graph_reverse(target, &mut graph).await;
        }

        Ok(graph)
    }
}
