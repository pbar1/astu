use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::action::AuthPayload;
use crate::action::Client;
use crate::action::ClientFactory;
use crate::action::ClientImpl;
use crate::action::ExecOutput;
use crate::action::ExecRequest;
use crate::resolve::Target;
use crate::resolve::TargetKind;

#[derive(Debug, Clone, Default)]
pub struct DummyClientFactory;

impl ClientFactory for DummyClientFactory {
    fn client(&self, target: &Target) -> Option<ClientImpl> {
        if target.kind() != TargetKind::Dummy {
            return None;
        }
        Some(DummyClient::new(target).into())
    }
}

pub struct DummyClient {
    target: Target,
}

impl DummyClient {
    #[must_use]
    pub fn new(target: &Target) -> Self {
        Self {
            target: target.to_owned(),
        }
    }
}

#[async_trait]
impl Client for DummyClient {
    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn ping(&mut self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    async fn auth(&mut self, _auth_type: &AuthPayload) -> Result<()> {
        Ok(())
    }

    async fn exec(&mut self, _command: &str, request: ExecRequest) -> Result<ExecOutput> {
        let query = self.target.query_pairs().unwrap_or_default();

        let stdout = query
            .get("stdout")
            .cloned()
            .unwrap_or_default()
            .into_bytes();
        let stderr = query
            .get("stderr")
            .cloned()
            .unwrap_or_default()
            .into_bytes();
        let exit_status = query
            .get("exitcode")
            .map(|x| x.parse::<u32>())
            .transpose()
            .map_err(|error| anyhow!("invalid dummy exitcode: {error}"))?
            .unwrap_or(0);

        if request.live {
            if !stdout.is_empty() {
                tokio::io::stdout().write_all(&stdout).await?;
            }
            if !stderr.is_empty() {
                tokio::io::stderr().write_all(&stderr).await?;
            }
        }

        Ok(ExecOutput {
            exit_status,
            stdout,
            stderr,
        })
    }
}
