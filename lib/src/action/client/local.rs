use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::action::AuthPayload;
use crate::action::Client;
use crate::action::ClientFactory;
use crate::action::ClientImpl;
use crate::action::ExecOutput;
use crate::action::ExecRequest;
use crate::action::ExecStdin;
use crate::resolve::Target;
use crate::resolve::TargetKind;

const SPOOL_DRAIN_LOG_EVERY_BYTES: u64 = 64 * 1024 * 1024;

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

    async fn exec(&mut self, command: &str, request: ExecRequest) -> Result<ExecOutput> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        detach_from_terminal_signal_group(&mut cmd);
        let mut child = cmd.spawn().context("unable to spawn local process")?;

        let stdin_req = request.stdin.clone();
        let live = request.live;
        let mut child_stdin = child.stdin.take();
        let stdin_task = tokio::spawn(async move {
            if let Some(mut stdin) = child_stdin.take() {
                match stdin_req {
                    Some(ExecStdin::Bytes(stdin_buf)) => {
                        if let Err(error) = stdin.write_all(&stdin_buf).await {
                            if !is_broken_pipe(&error) {
                                return Err(error).context("unable to write local stdin bytes");
                            }
                            return Ok(());
                        }
                    }
                    Some(ExecStdin::SpoolFile { path, done_path }) => {
                        tracing::debug!(
                            spool_path = %path.display(),
                            done_path = %done_path.display(),
                            "starting spool drain for local exec",
                        );
                        let mut file = tokio::fs::File::open(&path)
                            .await
                            .context("unable to open spool file")?;
                        let mut buf = [0_u8; 16 * 1024];
                        let mut consumed = 0_u64;
                        let mut last_log_bucket = 0_u64;
                        loop {
                            let n = file
                                .read(&mut buf)
                                .await
                                .context("unable to read spool file")?;
                            if n == 0 {
                                let done = tokio::fs::try_exists(&done_path)
                                    .await
                                    .context("unable to stat spool done file")?;
                                if done {
                                    tracing::debug!(
                                        spool_path = %path.display(),
                                        consumed_bytes = consumed,
                                        "spool drain reached EOF with done marker",
                                    );
                                    break;
                                }
                                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                                continue;
                            }
                            consumed =
                                consumed.saturating_add(u64::try_from(n).unwrap_or(u64::MAX));
                            if let Err(error) = stdin.write_all(&buf[..n]).await {
                                if !is_broken_pipe(&error) {
                                    return Err(error).context("unable to write local stdin stream");
                                }
                                return Ok(());
                            }
                            let bucket = consumed / SPOOL_DRAIN_LOG_EVERY_BYTES;
                            if bucket > last_log_bucket {
                                last_log_bucket = bucket;
                                let head_bytes = tokio::fs::metadata(&path)
                                    .await
                                    .map(|m| m.len())
                                    .unwrap_or(consumed);
                                let lag_bytes = head_bytes.saturating_sub(consumed);
                                tracing::debug!(
                                    spool_path = %path.display(),
                                    consumed_bytes = consumed,
                                    head_bytes,
                                    lag_bytes,
                                    "spool drain progress",
                                );
                            }
                        }
                    }
                    None => {}
                }
                if let Err(error) = stdin.shutdown().await {
                    if !is_broken_pipe(&error) {
                        return Err(error).context("unable to close local stdin");
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        let stdout_task = tokio::spawn(read_child_stream(child.stdout.take(), live, false));
        let stderr_task = tokio::spawn(read_child_stream(child.stderr.take(), live, true));

        let status = child
            .wait()
            .await
            .context("unable to wait for local process")?;
        stdin_task.await.context("join stdin task")??;
        let stdout = stdout_task.await.context("join stdout task")??;
        let stderr = stderr_task.await.context("join stderr task")??;

        let exit_status = u32::try_from(status.code().unwrap_or(1)).unwrap_or(1);

        Ok(ExecOutput {
            exit_status,
            stdout,
            stderr,
        })
    }
}

fn detach_from_terminal_signal_group(_cmd: &mut Command) {
    #[cfg(unix)]
    unsafe {
        _cmd.pre_exec(|| {
            let rc = libc::setpgid(0, 0);
            if rc == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
}

fn is_broken_pipe(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::BrokenPipe
}

async fn read_child_stream(
    reader: Option<impl tokio::io::AsyncRead + Unpin>,
    live: bool,
    stderr: bool,
) -> Result<Vec<u8>> {
    let Some(mut reader) = reader else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    let mut buf = [0_u8; 16 * 1024];
    loop {
        let n = reader.read(&mut buf).await.context("read local stream")?;
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
        if live {
            if stderr {
                tokio::io::stderr()
                    .write_all(&buf[..n])
                    .await
                    .context("stream stderr")?;
            } else {
                tokio::io::stdout()
                    .write_all(&buf[..n])
                    .await
                    .context("stream stdout")?;
            }
        }
    }
    Ok(out)
}
