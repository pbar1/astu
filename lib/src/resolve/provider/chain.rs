use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use async_stream::stream;
use futures::stream::BoxStream;
use futures::StreamExt;

use crate::resolve::Resolve;
use crate::resolve::Target;

/// Composite resolver that flattens the streams of a set of resolvers into one.
///
/// If none of the constituent resolvers can resolve a given target, that target
/// itself is returned.
#[derive(Clone)]
pub struct ChainResolver {
    resolvers: Vec<Arc<dyn Resolve + Send + Sync>>,
}

impl fmt::Debug for ChainResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChainResolver").finish()
    }
}

impl Resolve for ChainResolver {
    fn resolve_fallible(&self, target: Target) -> BoxStream<Result<Target>> {
        stream! {
            let mut bounce_original = true;
            for resolver in &self.resolvers {
                let mut stream = resolver.resolve_fallible(target.clone());
                while let Some(result) = stream.next().await {
                    bounce_original = false;
                    yield result;
                }
            }
            if bounce_original {
                yield Ok(target);
            }
        }
        .boxed()
    }
}

impl ChainResolver {
    pub fn new() -> Self {
        ChainResolver {
            resolvers: Vec::new(),
        }
    }

    pub fn with(mut self, resolver: impl Resolve + Send + Sync + 'static) -> Self {
        self.resolvers.push(Arc::new(resolver));
        self
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::resolve::CidrResolver;
    use crate::resolve::DnsResolver;
    use crate::resolve::ResolveExt;

    #[rstest]
    #[case("127.0.0.1", 1)]
    #[case("127.0.0.1/31", 2)]
    #[case("localhost", 1)]
    #[tokio::test]
    async fn resolve_works(#[case] query: &str, #[case] num: usize) {
        let target = Target::from_str(query).unwrap();
        let resolver = ChainResolver::new()
            .with(CidrResolver::new())
            .with(DnsResolver::try_new().unwrap());
        let targets = resolver.resolve_set(target).await;
        dbg!(&targets);
        assert_eq!(targets.len(), num);
    }
}
