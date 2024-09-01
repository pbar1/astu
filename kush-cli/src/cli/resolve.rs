use anyhow::bail;
use clap::Args;
use kush_resolve::Resolve;

/// Resolve targets from queries
#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Target query
    query: String,
}

#[async_trait::async_trait]
impl super::Run for ResolveArgs {
    async fn run(&self) -> anyhow::Result<()> {
        let mut resolvers: Vec<Box<dyn Resolve + Send + Sync>> = Vec::new();
        resolvers.push(Box::new(kush_resolve::IpResolver));
        resolvers.push(Box::new(kush_resolve::DnsResolver::system()?));

        let mut targets = None;
        for resolver in resolvers {
            if let Ok(t) = resolver.resolve(&self.query).await {
                targets = Some(t);
                break;
            };
        }
        let Some(targets) = targets else {
            bail!("no targets found for query: {}", &self.query);
        };

        for target in targets {
            println!("{target}");
        }

        Ok(())
    }
}
