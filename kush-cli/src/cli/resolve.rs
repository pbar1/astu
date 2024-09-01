use std::collections::BTreeSet;

use anyhow::bail;
use clap::Args;
use kush_resolve::Resolve;
use kush_resolve::ReverseResolve;
use kush_resolve::Target;

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
            let mut resolvers: Vec<Box<dyn ReverseResolve + Send + Sync>> = Vec::new();
            resolvers.push(Box::new(kush_resolve::DnsResolver::system()?));
            reverse_resolve(resolvers, &self.query).await
        } else {
            let mut resolvers: Vec<Box<dyn Resolve + Send + Sync>> = Vec::new();
            resolvers.push(Box::new(kush_resolve::IpResolver));
            resolvers.push(Box::new(kush_resolve::DnsResolver::system()?));
            resolve(resolvers, &self.query).await
        };

        let Some(targets) = targets else {
            bail!("no targets found for query: {}", &self.query);
        };

        for target in targets {
            println!("{target}");
        }

        Ok(())
    }
}

async fn resolve(
    resolvers: Vec<Box<dyn Resolve + Send + Sync>>,
    query: &str,
) -> Option<BTreeSet<Target>> {
    let mut targets = None;
    for resolver in resolvers {
        if let Ok(t) = resolver.resolve(query).await {
            targets = Some(t);
            break;
        };
    }
    targets
}
async fn reverse_resolve(
    resolvers: Vec<Box<dyn ReverseResolve + Send + Sync>>,
    query: &str,
) -> Option<BTreeSet<Target>> {
    let mut targets = None;
    for resolver in resolvers {
        if let Ok(t) = resolver.reverse_resolve(query).await {
            targets = Some(t);
            break;
        };
    }
    targets
}
