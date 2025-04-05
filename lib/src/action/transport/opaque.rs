use anyhow::Result;
use async_trait::async_trait;

use crate::resolve::Target;

#[derive(Debug, Default, Clone, Copy)]
pub struct TransportFactory {}

#[async_trait]
impl super::TransportFactory for TransportFactory {
    async fn setup(&self, _target: &Target) -> Result<super::Transport> {
        Ok(super::Transport::Opaque)
    }
}
