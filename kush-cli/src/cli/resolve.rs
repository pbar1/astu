use std::str::FromStr;

use async_stream::stream;
use clap::Args;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use kush_resolve::ForwardResolveChain;
use kush_resolve::Resolve;
use kush_resolve::ReverseResolveChain;
use kush_resolve::Target;
use tokio::io::AsyncBufReadExt;

/// Resolve targets from queries
#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Target query. Mutually exclusive with TARGETS_FILE.
    target: Option<Target>,

    /// Targets file. Mutually exclusive with TARGET.
    targets_file: Option<String>,

    /// Perform reverse resolution instead of forward
    #[arg(short, long)]
    reverse: bool,

    /// Show targets that resolve to unknown
    #[arg(long)]
    show_unknown: bool,
}

#[async_trait::async_trait]
impl super::Run for ResolveArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let tstream = stream! {
            if let Some(target) = self.target.clone() {
                yield target;
            }
            if let Some(tfile) = self.targets_file.clone() {
                for await target in stream_targets(tfile) {
                    yield target;
                }
            }
        };
        pin_mut!(tstream);

        if self.reverse {
            let resolvers = ReverseResolveChain::try_default()?;
            while let Some(target) = tstream.next().await {
                let targets = resolvers.resolve(target);
                process_targets(targets, self.show_unknown).await;
            }
        } else {
            let resolvers = ForwardResolveChain::try_default()?;
            while let Some(target) = tstream.next().await {
                let targets = resolvers.resolve(target);
                process_targets(targets, self.show_unknown).await;
            }
        }

        Ok(())
    }
}

fn stream_targets(file: String) -> impl Stream<Item = Target> {
    stream! {
        let file = tokio::fs::File::open(file).await.unwrap();
        let reader = tokio::io::BufReader::new(file);
        let lines = tokio_stream::wrappers::LinesStream::new(reader.lines());
        for await line in lines {
            let Ok(line) = line else {
                continue;
            };
            if let Ok(t) = Target::from_str(&line.trim()) {
                yield t;
            }
        }
    }
}

async fn process_targets(targets: impl Stream<Item = Target>, ignore_unknown: bool) {
    pin_mut!(targets);
    while let Some(target) = targets.next().await {
        if ignore_unknown && target.is_unknown() {
            continue;
        }
        println!("{target}");
    }
}
