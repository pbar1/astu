use std::sync::Arc;

use astu_resolve::Target;

use super::Client;
use super::ClientFactory;

/// Composite factory for mapping targets to clients at runtime.
///
/// Constituent factories will be iterated until one can build a client.
#[derive(Clone)]
pub struct DynamicClientFactory {
    factories: Vec<Arc<dyn ClientFactory + Send + Sync>>,
}

impl DynamicClientFactory {
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }

    pub fn with(mut self, factory: impl ClientFactory + Send + Sync + 'static) -> Self {
        self.factories.push(Arc::new(factory));
        self
    }
}

impl ClientFactory for DynamicClientFactory {
    fn client(&self, target: &Target) -> Option<Client> {
        for factory in &self.factories {
            if let Some(client) = factory.client(target) {
                return Some(client);
            }
        }
        None
    }
}
