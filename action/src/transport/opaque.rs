use anyhow::Result;
use astu_resolve::Target;
use async_trait::async_trait;

#[derive(Debug, Clone, Copy)]
pub struct TransportFactory {}

impl TransportFactory {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl super::TransportFactory for TransportFactory {
    async fn setup(&self, _target: &Target) -> Result<super::Transport> {
        Ok(super::Transport::Opaque)
    }
}
