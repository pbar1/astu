use std::str::FromStr;

use async_stream::stream;
use fluent_uri::UriRef;
use futures::Stream;

use crate::Resolve;
use crate::Target;

pub struct UriResolver;

impl Resolve for UriResolver {
    fn resolve(&self, target: Target) -> impl Stream<Item = Target> {
        stream! {
            match target {
                Target::Uri(x) => yield map_uri(x),
                _unsupported => return,
            };
        }
    }
}

fn map_uri(uri: UriRef<String>) -> Target {
    match uri.scheme().map(|x| x.as_str()) {
        Some("ssh") => ssh(uri),
        _unknown => Target::Unknown(uri.to_string()),
    }
}

fn ssh(uri: UriRef<String>) -> Target {
    let Some(authority) = uri.authority() else {
        return Target::Unknown(uri.to_string());
    };
    let s = match authority.port_to_u16() {
        Ok(Some(_)) => authority.as_str(),
        Ok(None) => &format!("{authority}:22"),
        _else => return Target::Unknown(uri.to_string()),
    };
    let Ok(target) = Target::from_str(&s) else {
        return Target::Unknown(uri.to_string());
    };
    target
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::str::FromStr;

    use futures::StreamExt;
    use rstest::rstest;

    use super::*;
    use crate::Resolve;

    #[rstest]
    #[case("ssh://127.0.0.1", "127.0.0.1:22")]
    #[case("ssh://127.0.0.1:22", "127.0.0.1:22")]
    #[case("ssh://[::1]", "[::1]:22")]
    #[case("ssh://[::1]:22", "[::1]:22")]
    #[tokio::test]
    async fn resolve_works(#[case] query: &str, #[case] should: &str) {
        let target = Target::from_str(query).unwrap();
        let resolver = UriResolver;
        let targets: BTreeSet<Target> = resolver.resolve(target).collect().await;
        assert_eq!(targets.len(), 1);
        assert_eq!(targets.first().unwrap().to_string(), should);
    }
}
