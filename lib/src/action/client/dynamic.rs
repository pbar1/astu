use crate::action::ClientFactoryImpl;
use crate::action::ClientImpl;
use crate::resolve::Target;

/// Composite factory for mapping targets to clients at runtime.
///
/// Constituent factories will be iterated until one can build a client.
#[derive(Clone)]
pub struct DynamicClientFactory {
    factories: Vec<ClientFactoryImpl>,
}

impl DynamicClientFactory {
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }

    pub fn with(mut self, factory: impl Into<ClientFactoryImpl>) -> Self {
        self.factories.push(factory.into());
        self
    }
}

impl crate::action::ClientFactory for DynamicClientFactory {
    fn client(&self, target: &Target) -> Option<ClientImpl> {
        for factory in &self.factories {
            if let Some(client) = factory.client(target) {
                return Some(client);
            }
        }
        None
    }
}
