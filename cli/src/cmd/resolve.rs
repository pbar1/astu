use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use clap::ValueEnum;

use crate::cmd::Run;

/// Resolve targets
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[clap(flatten)]
    resolution_args: crate::args::ResolutionArgs,

    /// Output mode
    #[clap(short = 'm', long, value_enum, default_value_t = OutputMode::default())]
    mode: OutputMode,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum OutputMode {
    #[default]
    Targets,
    Buckets,
    Graph,
}

impl Run for ResolveArgs {
    async fn run(&self, _id: Id, _db: DbImpl) -> Result<()> {
        let res_args = self.resolution_args.clone();

        match self.mode {
            OutputMode::Targets => {
                for target in res_args.set().await? {
                    println!("{target}");
                }
            }
            OutputMode::Buckets => {
                let buckets = res_args.graph_full().await?.buckets();
                for (group, targets) in buckets {
                    println!("{group}");
                    for target in targets {
                        println!("\t{target}");
                    }
                }
            }
            OutputMode::Graph => {
                let graphviz = res_args.graph_full().await?.graphviz();
                println!("{graphviz}");
            }
        }

        Ok(())
    }
}
