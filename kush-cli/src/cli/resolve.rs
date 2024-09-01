use clap::Args;
use kush_resolve::Resolve;
use kush_resolve::ResolveChain;
use kush_resolve::ReverseResolve;
use kush_resolve::ReverseResolveChain;

/// Resolve targets from queries
#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Target query
    query: String,

    /// Perform reverse resolution instead of forward
    #[arg(short, long)]
    reverse: bool,
}

#[async_trait::async_trait]
impl super::Run for ResolveArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let targets = if self.reverse {
            let resolvers = ReverseResolveChain::try_default()?;
            resolvers.reverse_resolve(&self.query).await?
        } else {
            let resolvers = ResolveChain::forward()?;
            resolvers.resolve(&self.query).await?
        };

        for target in targets {
            println!("{target}");
        }

        Ok(())
    }
}
