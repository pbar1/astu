use clap::Args;
use futures::pin_mut;
use futures::StreamExt;
use kush_resolve::ForwardResolveChain;
use kush_resolve::Resolve;
use kush_resolve::ReverseResolveChain;
use kush_resolve::Target;

/// Resolve targets from queries
#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Target query
    query: Target,

    /// Perform reverse resolution instead of forward
    #[arg(short, long)]
    reverse: bool,
}

#[async_trait::async_trait]
impl super::Run for ResolveArgs {
    async fn run(&self) -> anyhow::Result<()> {
        if self.reverse {
            let resolvers = ReverseResolveChain::try_default()?;
            let targets = resolvers.resolve(self.query.clone());
            pin_mut!(targets);
            while let Some(target) = targets.next().await {
                println!("{target}");
            }
        } else {
            let resolvers = ForwardResolveChain::try_default()?;
            let targets = resolvers.resolve(self.query.clone());
            pin_mut!(targets);
            while let Some(target) = targets.next().await {
                println!("{target}");
            }
        }

        Ok(())
    }
}
