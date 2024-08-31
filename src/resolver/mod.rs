use async_trait::async_trait;
use thiserror::Error;

use crate::target::Target;

pub mod dns;

/// Errors regarding [`Resolver`].
#[derive(Debug, Clone, Error)]
pub enum ResolverError {
    #[error("unknown resolver failure: {0}")]
    Unknown(String),

    #[error("no targets found")]
    NoTargets,

    #[error("timeout during resolution")]
    Timeout,
}

#[async_trait]
pub trait Resolver {
    async fn resolve(&self, search_term: &str) -> Result<Vec<Target>, ResolverError>;
}
