use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;

use super::Transport;
use super::TransportFactory;

#[derive(Debug, Clone, Copy)]
pub struct OpaqueTransportFactory {}

impl OpaqueTransportFactory {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TransportFactory for OpaqueTransportFactory {
    async fn connect(&self, _target: &Target) -> Result<Transport> {
        Ok(Transport::Opaque)
    }
}
