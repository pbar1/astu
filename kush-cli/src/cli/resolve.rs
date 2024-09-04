use clap::Args;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use kush_resolve::ForwardResolveChain;
use kush_resolve::Resolve;
use kush_resolve::ReverseResolveChain;
use kush_resolve::Target;

/// Resolve targets from queries
#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Target query. Reads from stdin by default.
    #[arg(default_value = "/dev/fd/0")]
    target: Target,

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
        if self.reverse {
            let resolvers = ReverseResolveChain::try_default()?;
            let targets = resolvers.resolve(self.target.clone());
            process_targets(targets, self.show_unknown).await;
        } else {
            let resolvers = ForwardResolveChain::try_default()?;
            let targets = resolvers.resolve(self.target.clone());
            process_targets(targets, self.show_unknown).await;
        }
        Ok(())
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
