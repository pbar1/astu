use anyhow::Result;
use astu::action::client;
use astu::action::transport;
use astu::action::AuthPayload;
use astu::action::Client;
use astu::action::ClientFactory;
use astu::action::ExecRequest;
use astu::action::ExecStdin;
use astu::db::DbTaskStatus;
use astu::db::DuckDb;
use astu::normalize::Normalizer;
use astu::resolve::Target;
use clap::Args;
use clap::ValueEnum;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::Instrument;
use uuid::Uuid;

use super::AuthArgs;

const HEADING: Option<&str> = Some("Action Options");

/// Arguments for action execution.
#[derive(Debug, Args, Clone)]
pub struct ActionArgs {
    /// Number of actions to process at once.
    #[clap(long, default_value_t = 500, help_heading = HEADING)]
    pub concurrency: usize,

    /// Confirm target count
    #[clap(long, help_heading = HEADING)]
    pub confirm: Option<usize>,

    /// Time to allow each action to complete.
    #[clap(long, default_value = "30s", help_heading = HEADING)]
    pub timeout: humantime::Duration,

    /// How to interpret stdin.
    #[clap(long, default_value_t = StdinMode::default(), value_enum, help_heading = HEADING)]
    pub stdin: StdinMode,

    /// Stream task stdout/stderr to terminal while running.
    #[clap(long, default_value_t = false, help_heading = HEADING)]
    pub live: bool,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum StdinMode {
    #[default]
    Auto,
    Param,
    Target,
    Pipe,
}

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

pub type PreparedStdin = crate::action::stdin::PreparedStdin;

#[derive(Debug, Clone)]
pub enum ActionOperation {
    Exec {
        auth: AuthArgs,
        pipe_stdin: Option<crate::action::stdin::PipeSpool>,
        live: bool,
    },
    Ping,
}

#[derive(Debug)]
struct SpoolCleanup(Option<crate::action::stdin::PipeSpool>);

impl Drop for SpoolCleanup {
    fn drop(&mut self) {
        if let Some(spool) = self.0.take() {
            let _ = std::fs::remove_file(&spool.path);
            let _ = std::fs::remove_file(&spool.done_path);
        }
    }
}

impl ActionArgs {
    pub fn client_factory(&self) -> Result<client::DynamicClientFactory> {
        // Transports
        let t_tcp = transport::tcp_reuse::TransportFactory::try_new(self.timeout.into())?;

        // Clients
        let c_ssh = client::SshClientFactory::new(t_tcp.clone().into());
        let c_tcp = client::TcpClientFactory::new(t_tcp.clone().into());
        let c_local = client::LocalClientFactory;
        let c_dummy = client::DummyClientFactory;

        // Mapper
        let mapper = client::DynamicClientFactory::default()
            .with(c_local)
            .with(c_dummy)
            .with(c_ssh)
            .with(c_tcp);
        Ok(mapper)
    }

    pub fn infer_input_mode(&self, command: &str, has_stdin_target_file: bool) -> InputMode {
        match self.stdin {
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

    pub fn require_confirm(&self, target_count: usize) -> Result<()> {
        crate::action::confirm::require_confirm(self.confirm, target_count)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn run_tasks(
        &self,
        db: DuckDb,
        job_id: &str,
        specs: Vec<TaskSpec>,
        auth: &AuthArgs,
        pipe_stdin: Option<crate::action::stdin::PipeSpool>,
    ) -> Result<()> {
        self.run_tasks_for_operation(
            db,
            job_id,
            specs,
            ActionOperation::Exec {
                auth: auth.clone(),
                pipe_stdin,
                live: self.live,
            },
        )
        .await
    }

    #[allow(clippy::too_many_lines)]
    pub async fn run_tasks_for_operation(
        &self,
        db: DuckDb,
        job_id: &str,
        specs: Vec<TaskSpec>,
        operation: ActionOperation,
    ) -> Result<()> {
        let _spool_cleanup = match &operation {
            ActionOperation::Exec { pipe_stdin, .. } => SpoolCleanup(pipe_stdin.clone()),
            ActionOperation::Ping => SpoolCleanup(None),
        };
        let command = match &operation {
            ActionOperation::Ping => "astu ping".to_owned(),
            ActionOperation::Exec { .. } => {
                specs.first().map(|x| x.command.clone()).unwrap_or_default()
            }
        };
        db.create_job(
            job_id,
            &command,
            i64::try_from(self.concurrency).unwrap_or(i64::MAX),
            i64::try_from(specs.len()).unwrap_or(i64::MAX),
        )
        .await?;

        let cancel = Arc::new(AtomicBool::new(false));
        {
            let cancel = cancel.clone();
            tokio::spawn(async move {
                let mut interrupts = 0_u8;
                loop {
                    let _ = tokio::signal::ctrl_c().await;
                    interrupts = interrupts.saturating_add(1);
                    if interrupts == 1 {
                        eprintln!(
                            "Received interrupt. Stopping new task starts (press Ctrl-C again to force exit)."
                        );
                        cancel.store(true, Ordering::SeqCst);
                        continue;
                    }
                    eprintln!("Received second interrupt. Forcing exit.");
                    std::process::exit(130);
                }
            });
        }

        let sem = Arc::new(Semaphore::new(self.concurrency.max(1)));
        let client_factory = self.client_factory()?;
        let mut tasks = tokio::task::JoinSet::new();

        for spec in specs {
            let task_id = Uuid::now_v7().hyphenated().to_string();
            if cancel.load(Ordering::SeqCst) {
                db.create_task(&task_id, job_id, &spec.target.to_string(), &spec.command)
                    .await?;
                crate::action::template::append_task_template_vars(
                    &db,
                    &task_id,
                    &spec.target,
                    spec.param.as_deref(),
                )
                .await?;
                db.finish_task(
                    &task_id,
                    DbTaskStatus::Canceled,
                    None,
                    Some("canceled by interrupt"),
                    0,
                    0,
                    0,
                )
                .await?;
                continue;
            }

            let permit = sem.clone().acquire_owned().await?;
            db.create_task(&task_id, job_id, &spec.target.to_string(), &spec.command)
                .await?;
            crate::action::template::append_task_template_vars(
                &db,
                &task_id,
                &spec.target,
                spec.param.as_deref(),
            )
            .await?;

            let db = db.clone();
            let client_factory = client_factory.clone();
            let operation = operation.clone();

            let target_uri = spec.target.to_string();
            let task_span = match &operation {
                ActionOperation::Exec { .. } => tracing::info_span!("exec", %target_uri),
                ActionOperation::Ping => tracing::info_span!("ping", %target_uri),
            };
            tasks.spawn(
                async move {
                    let _permit = permit;
                    let Some(mut client) = client_factory.client(&spec.target) else {
                        finish_task_failed(
                            &db,
                            &task_id,
                            "failed getting client for target".to_owned(),
                            0,
                            0,
                            0,
                        )
                        .await?;
                        return Ok::<(), anyhow::Error>(());
                    };

                    let t_connect = Instant::now();
                    let connect_result = client.connect().await;
                    let connect_ms =
                        i64::try_from(t_connect.elapsed().as_millis()).unwrap_or(i64::MAX);
                    if let Err(error) = connect_result {
                        finish_task_failed(&db, &task_id, format!("{error:?}"), connect_ms, 0, 0)
                            .await?;
                        return Ok::<(), anyhow::Error>(());
                    }

                    match operation {
                        ActionOperation::Exec {
                            auth,
                            pipe_stdin,
                            live,
                        } => {
                            let t_auth = Instant::now();
                            if let Some(user) = spec.target.user().or(Some(auth.user.as_str())) {
                                if let Err(error) =
                                    client.auth(&AuthPayload::User(user.to_string())).await
                                {
                                    finish_task_failed(
                                        &db,
                                        &task_id,
                                        format!("{error:?}"),
                                        connect_ms,
                                        i64::try_from(t_auth.elapsed().as_millis())
                                            .unwrap_or(i64::MAX),
                                        0,
                                    )
                                    .await?;
                                    return Ok::<(), anyhow::Error>(());
                                }
                            }
                            if let Some(socket) = auth.ssh_agent {
                                if !std::path::Path::new(socket.as_str()).exists() {
                                    finish_task_failed(
                                        &db,
                                        &task_id,
                                        format!("ssh-agent socket does not exist: {socket}"),
                                        connect_ms,
                                        i64::try_from(t_auth.elapsed().as_millis())
                                            .unwrap_or(i64::MAX),
                                        0,
                                    )
                                    .await?;
                                    return Ok::<(), anyhow::Error>(());
                                }
                                if let Err(error) = client
                                    .auth(&AuthPayload::SshAgent {
                                        socket: socket.to_string(),
                                    })
                                    .await
                                {
                                    finish_task_failed(
                                        &db,
                                        &task_id,
                                        format!("{error:?}"),
                                        connect_ms,
                                        i64::try_from(t_auth.elapsed().as_millis())
                                            .unwrap_or(i64::MAX),
                                        0,
                                    )
                                    .await?;
                                    return Ok::<(), anyhow::Error>(());
                                }
                            }
                            let auth_ms =
                                i64::try_from(t_auth.elapsed().as_millis()).unwrap_or(i64::MAX);
                            let stdin_input = pipe_stdin.map(|spool| ExecStdin::SpoolFile {
                                path: spool.path,
                                done_path: spool.done_path,
                            });

                            let t_exec = Instant::now();
                            let result = client
                                .exec(
                                    &spec.command,
                                    ExecRequest {
                                        stdin: stdin_input,
                                        live,
                                    },
                                )
                                .await;
                            let exec_ms =
                                i64::try_from(t_exec.elapsed().as_millis()).unwrap_or(i64::MAX);

                            match result {
                                Ok(output) => {
                                    let normalizer = Normalizer::from_token_values(
                                        crate::action::template::task_template_values(
                                            &spec.target,
                                            spec.param.as_deref(),
                                        ),
                                    );
                                    let stdout = crate::action::template::normalize_stream_bytes(
                                        &normalizer,
                                        &output.stdout,
                                    );
                                    let stderr = crate::action::template::normalize_stream_bytes(
                                        &normalizer,
                                        &output.stderr,
                                    );

                                    db.append_stream_blob(&task_id, "stdout", &stdout).await?;
                                    db.append_stream_blob(&task_id, "stderr", &stderr).await?;
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
                                    finish_task_failed(
                                        &db,
                                        &task_id,
                                        format!("{error:?}"),
                                        connect_ms,
                                        auth_ms,
                                        exec_ms,
                                    )
                                    .await?;
                                }
                            }
                        }
                        ActionOperation::Ping => {
                            let t_ping = Instant::now();
                            match client.ping().await {
                                Ok(stdout) => {
                                    db.append_stream_blob(&task_id, "stdout", &stdout).await?;
                                    db.finish_task(
                                        &task_id,
                                        DbTaskStatus::Complete,
                                        Some(0),
                                        None,
                                        connect_ms,
                                        0,
                                        i64::try_from(t_ping.elapsed().as_millis())
                                            .unwrap_or(i64::MAX),
                                    )
                                    .await?;
                                }
                                Err(error) => {
                                    finish_task_failed(
                                        &db,
                                        &task_id,
                                        format!("{error:?}"),
                                        connect_ms,
                                        0,
                                        i64::try_from(t_ping.elapsed().as_millis())
                                            .unwrap_or(i64::MAX),
                                    )
                                    .await?;
                                }
                            }
                        }
                    }

                    Ok::<(), anyhow::Error>(())
                }
                .instrument(task_span),
            );
        }

        while let Some(result) = tasks.join_next().await {
            result??;
        }

        db.finish_job(job_id).await?;
        Ok(())
    }
}

pub async fn read_stdin_for_mode(
    data_dir: &str,
    job_id: &str,
    mode: InputMode,
) -> Result<PreparedStdin> {
    crate::action::stdin::read_stdin_for_mode(data_dir, job_id, mode).await
}

pub fn build_task_specs(
    targets: Vec<Target>,
    command: &str,
    mode: InputMode,
    stdin: &[u8],
) -> Vec<TaskSpec> {
    crate::action::stdin::build_task_specs(targets, command, mode, stdin)
}

async fn finish_task_failed(
    db: &DuckDb,
    task_id: &str,
    error: String,
    connect_ms: i64,
    auth_ms: i64,
    exec_ms: i64,
) -> Result<()> {
    db.finish_task(
        task_id,
        DbTaskStatus::Failed,
        None,
        Some(&error),
        connect_ms,
        auth_ms,
        exec_ms,
    )
    .await
}
