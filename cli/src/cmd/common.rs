use std::collections::BTreeSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use astu::action::AuthPayload;
use astu::action::Client;
use astu::action::ClientFactory;
use astu::db::DbImpl;
use astu::db::DbTaskStatus;
use astu::resolve::Host;
use astu::resolve::Target;
use tokio::io::AsyncReadExt;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::args::ActionArgs;
use crate::args::AuthArgs;
use crate::args::StdinMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Param,
    Target,
    Pipe,
}

#[derive(Debug, Clone)]
pub struct TaskSpec {
    pub target: Target,
    pub command: String,
    pub param: Option<String>,
}

pub async fn read_stdin_all_if_piped() -> Result<Option<Vec<u8>>> {
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Ok(None);
    }

    let mut buf = Vec::new();
    tokio::io::stdin().read_to_end(&mut buf).await?;
    Ok(Some(buf))
}

pub fn infer_input_mode(
    action: &ActionArgs,
    command: &str,
    has_stdin_target_file: bool,
) -> InputMode {
    match action.stdin {
        StdinMode::Param => InputMode::Param,
        StdinMode::Target => InputMode::Target,
        StdinMode::Pipe => InputMode::Pipe,
        StdinMode::Auto => {
            if command.contains("{param}") {
                InputMode::Param
            } else if has_stdin_target_file {
                InputMode::Target
            } else {
                InputMode::Pipe
            }
        }
    }
}

pub fn normalize_targets(set: BTreeSet<Target>) -> Vec<Target> {
    if set.is_empty() {
        vec![Target::from_str("local:").expect("local target parses")]
    } else {
        set.into_iter().collect()
    }
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
                        command: render_command(command, &target, Some(param)),
                        param: Some(param.clone()),
                    });
                }
            }
            out
        }
        _ => targets
            .into_iter()
            .map(|target| TaskSpec {
                command: render_command(command, &target, None),
                target,
                param: None,
            })
            .collect(),
    }
}

pub fn render_command(template: &str, target: &Target, param: Option<&str>) -> String {
    let mut out = template.to_owned();

    if let Some(param) = param {
        out = out.replace("{param}", param);
    }

    if let Some(host) = target.host() {
        let host = match host {
            Host::Ip(ip) => ip.to_string(),
            Host::Domain(domain) => domain,
        };
        out = out.replace("{host}", &host);
    }

    if let Some(user) = target.user() {
        out = out.replace("{user}", user);
    }

    if let Some(ip) = target.ip() {
        out = out.replace("{ip}", &ip.to_string());
    }

    out
}

pub async fn run_tasks(
    db: DbImpl,
    job_id: &str,
    specs: Vec<TaskSpec>,
    auth: &AuthArgs,
    action: &ActionArgs,
    pipe_stdin: Option<PathBuf>,
) -> Result<()> {
    let db = match db {
        DbImpl::Duck(db) => db,
    };

    let command = specs.first().map(|x| x.command.clone()).unwrap_or_default();
    db.create_job(
        job_id,
        &command,
        i64::try_from(action.concurrency).unwrap_or(i64::MAX),
        i64::try_from(specs.len()).unwrap_or(i64::MAX),
    )
    .await?;

    let mut planned = Vec::new();
    for spec in specs {
        let task_id = Uuid::now_v7().hyphenated().to_string();
        db.create_task(&task_id, job_id, &spec.target.to_string(), &spec.command)
            .await?;
        if let Some(param) = &spec.param {
            db.append_task_var(&task_id, "{param}", param).await?;
        }
        planned.push((task_id, spec));
    }

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let cancel = cancel.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            cancel.store(true, Ordering::SeqCst);
        });
    }

    let sem = Arc::new(Semaphore::new(action.concurrency.max(1)));
    let client_factory = action.client_factory()?;
    let mut tasks = tokio::task::JoinSet::new();

    for (task_id, spec) in planned {
        if cancel.load(Ordering::SeqCst) {
            continue;
        }

        let permit = sem.clone().acquire_owned().await?;
        let db = db.clone();
        let client_factory = client_factory.clone();
        let auth = auth.clone();
        let pipe_stdin = pipe_stdin.clone();

        tasks.spawn(async move {
            let _permit = permit;

            let mut client = client_factory
                .client(&spec.target)
                .context("failed getting client for target")?;

            let t_connect = Instant::now();
            let connect_result = client.connect().await;
            let connect_ms = i64::try_from(t_connect.elapsed().as_millis()).unwrap_or(i64::MAX);
            if let Err(error) = connect_result {
                db.finish_task(
                    &task_id,
                    DbTaskStatus::Failed,
                    None,
                    Some(&format!("{error:#}")),
                    connect_ms,
                    0,
                    0,
                )
                .await?;
                return Ok::<(), anyhow::Error>(());
            }

            let t_auth = Instant::now();
            if let Some(user) = spec.target.user().or(Some(auth.user.as_str())) {
                let _ = client.auth(&AuthPayload::User(user.to_string())).await;
            }
            if let Some(socket) = auth.ssh_agent.clone() {
                let _ = client
                    .auth(&AuthPayload::SshAgent {
                        socket: socket.to_string(),
                    })
                    .await;
            }
            let auth_ms = i64::try_from(t_auth.elapsed().as_millis()).unwrap_or(i64::MAX);

            let stdin_bytes = if let Some(path) = pipe_stdin {
                std::fs::read(path).ok()
            } else {
                None
            };

            let t_exec = Instant::now();
            let result = client.exec(&spec.command, stdin_bytes.as_deref()).await;
            let exec_ms = i64::try_from(t_exec.elapsed().as_millis()).unwrap_or(i64::MAX);

            match result {
                Ok(output) => {
                    db.append_stream_blob(&task_id, "stdout", &output.stdout)
                        .await?;
                    db.append_stream_blob(&task_id, "stderr", &output.stderr)
                        .await?;
                    let status = if output.exit_status == 0 {
                        DbTaskStatus::Complete
                    } else {
                        DbTaskStatus::Failed
                    };
                    db.finish_task(
                        &task_id,
                        status,
                        Some(i64::from(output.exit_status)),
                        None,
                        connect_ms,
                        auth_ms,
                        exec_ms,
                    )
                    .await?;
                }
                Err(error) => {
                    db.finish_task(
                        &task_id,
                        DbTaskStatus::Failed,
                        None,
                        Some(&format!("{error:#}")),
                        connect_ms,
                        auth_ms,
                        exec_ms,
                    )
                    .await?;
                }
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(result) = tasks.join_next().await {
        result??;
    }

    db.finish_job(job_id).await?;
    Ok(())
}

pub fn maybe_spool_stdin(
    data_dir: &str,
    job_id: &str,
    mode: InputMode,
    stdin: &[u8],
) -> Result<Option<PathBuf>> {
    if mode != InputMode::Pipe || stdin.is_empty() {
        return Ok(None);
    }

    let spool_dir = PathBuf::from(data_dir).join("spool");
    std::fs::create_dir_all(&spool_dir)?;
    let path = spool_dir.join(format!("{job_id}.stdin"));
    std::fs::write(&path, stdin)?;
    Ok(Some(path))
}

pub fn require_confirm(confirm: Option<usize>, target_count: usize) -> Result<()> {
    let Some(confirm) = confirm else {
        bail!("--confirm={target_count} is required");
    };
    if confirm != target_count {
        bail!("--confirm expected {target_count}, got {confirm}");
    }
    Ok(())
}
