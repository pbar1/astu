use astu_types::Target;
use async_trait::async_trait;
use eyre::Result;

#[derive(Debug, Default, Clone, Copy)]
pub struct TransportFactory {}

#[async_trait]
impl super::TransportFactory for TransportFactory {
    async fn setup(&self, _target: &Target) -> Result<super::Transport> {
        Ok(super::Transport::Opaque)
    }
}
