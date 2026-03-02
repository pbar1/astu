use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::action::AuthPayload;
use crate::action::Client;
use crate::action::ClientFactory;
use crate::action::ClientImpl;
use crate::action::ExecOutput;
use crate::resolve::Target;
use crate::resolve::TargetKind;

#[derive(Debug, Clone, Default)]
pub struct LocalClientFactory;

impl ClientFactory for LocalClientFactory {
    fn client(&self, target: &Target) -> Option<ClientImpl> {
        if target.kind() != TargetKind::Local {
            return None;
        }
        Some(LocalClient::new(target).into())
    }
}

pub struct LocalClient {
    _target: Target,
    connected: bool,
}

impl LocalClient {
    #[must_use]
    pub fn new(target: &Target) -> Self {
        Self {
            _target: target.to_owned(),
            connected: false,
        }
    }
}

#[async_trait]
impl Client for LocalClient {
    async fn connect(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn ping(&mut self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    async fn auth(&mut self, _auth_type: &AuthPayload) -> Result<()> {
        Ok(())
    }

    async fn exec(&mut self, command: &str, stdin: Option<&[u8]>) -> Result<ExecOutput> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("unable to spawn local process")?;

        if let Some(stdin_buf) = stdin {
            if let Some(mut child_stdin) = child.stdin.take() {
                child_stdin
                    .write_all(stdin_buf)
                    .await
                    .context("unable to write local stdin")?;
                child_stdin
                    .shutdown()
                    .await
                    .context("unable to close local stdin")?;
            }
        }

        let output = child
            .wait_with_output()
            .await
            .context("unable to wait for local process")?;

        let exit_status = u32::try_from(output.status.code().unwrap_or(1)).unwrap_or(1);

        Ok(ExecOutput {
            exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}
