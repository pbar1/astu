use async_trait::async_trait;
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::config::ResolverOpts;
use hickory_resolver::error::ResolveError;
use hickory_resolver::error::ResolveErrorKind;
use hickory_resolver::TokioAsyncResolver;

use super::Resolver;
use super::ResolverError;
use crate::target::Target;

/// Resolves groups of [`Target`] by DNS.
pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl Default for DnsResolver {
    fn default() -> Self {
        let config = ResolverConfig::default();
        let options = ResolverOpts::default();
        let resolver = TokioAsyncResolver::tokio(config, options);

        Self { resolver }
    }
}

#[async_trait]
impl Resolver for DnsResolver {
    async fn resolve(&self, search_term: &str) -> Result<Vec<Target>, ResolverError> {
        // TODO: Default IP lookup strategy is `Ipv4thenIpv6`. Consider
        // changing it to `Ipv4AndIpv6` to gather all possible IPs.
        let targets = self
            .resolver
            .lookup_ip(search_term)
            .await
            .map_err(map_resolve_error)?
            .iter()
            .map(|ip| {
                // TODO: Don't force SSH from DNS resolution
                Target::Ssh {
                    host: ip.to_string(),
                    port: None,
                    user: None,
                }
            })
            .collect();

        Ok(targets)
    }
}

fn map_resolve_error(err: ResolveError) -> ResolverError {
    match err.kind() {
        ResolveErrorKind::Message(msg) => ResolverError::Unknown(msg.to_string()),
        ResolveErrorKind::Msg(msg) => ResolverError::Unknown(msg.to_owned()),
        ResolveErrorKind::NoRecordsFound { .. } => ResolverError::NoTargets,
        ResolveErrorKind::Timeout => ResolverError::Timeout,
        other => ResolverError::Unknown(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::dot("google.com.")]
    #[case::nodot("salesforce.com")]
    #[tokio::test]
    async fn dns_resolver_works(#[case] search_term: &str) {
        let resolver = DnsResolver::default();

        let targets = resolver.resolve(search_term).await.unwrap();

        dbg!(targets.clone());

        assert!(targets.len() > 0);
    }
}
