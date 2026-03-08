use astu_types::Target;

use crate::ClientFactoryImpl;
use crate::ClientImpl;

/// Composite factory for mapping targets to clients at runtime.
///
/// Constituent factories will be iterated until one can build a client.
#[derive(Default, Clone)]
pub struct DynamicClientFactory {
    factories: Vec<ClientFactoryImpl>,
}

impl DynamicClientFactory {
    #[must_use]
    pub fn with(mut self, factory: impl Into<ClientFactoryImpl>) -> Self {
        self.factories.push(factory.into());
        self
    }
}

impl crate::ClientFactory for DynamicClientFactory {
    fn client(&self, target: &Target) -> Option<ClientImpl> {
        for factory in &self.factories {
            if let Some(client) = factory.client(target) {
                return Some(client);
            }
        }
        None
    }
}
