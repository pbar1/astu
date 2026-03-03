use anyhow::Result;
use astu::resolve::Target;
use std::path::PathBuf;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::args::InputMode;
use crate::args::TaskSpec;

#[derive(Debug, Default)]
pub struct PreparedStdin {
    pub bytes: Vec<u8>,
    pub spool: Option<PipeSpool>,
}

#[derive(Debug, Clone)]
pub struct PipeSpool {
    pub path: PathBuf,
    pub done_path: PathBuf,
}

impl PipeSpool {
    pub fn new(data_dir: &str, job_id: &str) -> Self {
        let spool_dir = PathBuf::from(data_dir).join("spool");
        Self {
            path: spool_dir.join(format!("{job_id}.stdin")),
            done_path: spool_dir.join(format!("{job_id}.stdin.done")),
        }
    }
}

pub async fn read_stdin_for_mode(
    data_dir: &str,
    job_id: &str,
    mode: InputMode,
) -> Result<PreparedStdin> {
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Ok(PreparedStdin::default());
    }

    let mut stdin = tokio::io::stdin();
    read_reader_for_mode(&mut stdin, data_dir, job_id, mode).await
}

async fn read_reader_for_mode<R: AsyncRead + Unpin>(
    reader: &mut R,
    data_dir: &str,
    job_id: &str,
    mode: InputMode,
) -> Result<PreparedStdin> {
    if mode != InputMode::Pipe {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        return Ok(PreparedStdin { bytes, spool: None });
    }

    if job_id.is_empty() {
        anyhow::bail!("job_id is required for --stdin pipe spooling");
    }

    Ok(PreparedStdin {
        bytes: Vec::new(),
        spool: Some(PipeSpool::new(data_dir, job_id)),
    })
}

pub async fn pump_stdin_to_spool_with_cancel(
    spool: &PipeSpool,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<u64> {
    tokio::fs::create_dir_all(
        spool
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
    )
    .await?;
    let mut stdin = tokio::io::stdin();
    let mut file = tokio::fs::File::create(&spool.path).await?;
    let mut copied = 0_u64;
    let mut buf = [0_u8; 16 * 1024];
    loop {
        if *cancel_rx.borrow() {
            break;
        }
        let n = tokio::select! {
            changed = cancel_rx.changed() => {
                if changed.is_ok() && *cancel_rx.borrow() {
                    break;
                }
                continue;
            }
            read = stdin.read(&mut buf) => read?,
        };
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).await?;
        copied = copied.saturating_add(u64::try_from(n).unwrap_or(u64::MAX));
    }
    file.flush().await?;
    tokio::fs::write(&spool.done_path, b"done").await?;
    Ok(copied)
}

pub fn build_task_specs(
    targets: Vec<Target>,
    command: &str,
    mode: InputMode,
    stdin: &[u8],
) -> Vec<TaskSpec> {
    match mode {
        InputMode::Param => {
            let params = String::from_utf8_lossy(stdin)
                .split_whitespace()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();
            if params.is_empty() {
                return targets
                    .into_iter()
                    .map(|t| TaskSpec {
                        target: t,
                        command: command.to_owned(),
                        param: None,
                    })
                    .collect();
            }

            let mut out = Vec::new();
            for target in targets {
                for param in &params {
                    out.push(TaskSpec {
                        target: target.clone(),
                        command: crate::action::template::render_command(
                            command,
                            &target,
                            Some(param),
                        ),
                        param: Some(param.clone()),
                    });
                }
            }
            out
        }
        _ => targets
            .into_iter()
            .map(|target| TaskSpec {
                command: crate::action::template::render_command(command, &target, None),
                target,
                param: None,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use astu::resolve::Target;
    use tokio::io::BufReader;

    use super::build_task_specs;
    use super::read_reader_for_mode;
    use crate::args::InputMode;

    #[test]
    fn param_mode_expands_per_target_and_param() {
        let targets = vec![
            Target::from_str("dummy://a").expect("target"),
            Target::from_str("dummy://b").expect("target"),
        ];
        let specs = build_task_specs(targets, "echo {param}", InputMode::Param, b"x y");
        assert_eq!(specs.len(), 4);
    }

    #[tokio::test]
    async fn pipe_mode_prepares_spool_descriptor_without_buffering() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let mut reader = BufReader::new(std::io::Cursor::new(b"line-a\nline-b\n".to_vec()));

        let prepared = read_reader_for_mode(
            &mut reader,
            dir.path().to_str().expect("utf8 path"),
            "job-1",
            InputMode::Pipe,
        )
        .await
        .expect("prepared");

        assert!(prepared.bytes.is_empty(), "pipe mode must not keep bytes");
        let spool = prepared.spool.expect("spool path");
        assert!(spool.path.ends_with("job-1.stdin"));
        assert!(spool.done_path.ends_with("job-1.stdin.done"));
    }

}
