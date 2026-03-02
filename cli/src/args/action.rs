use anyhow::Result;
use anyhow::{anyhow, bail, Context};
use astu::action::client;
use astu::action::transport;
use astu::action::AuthPayload;
use astu::action::Client;
use astu::action::ClientFactory;
use astu::action::ExecStdin;
use astu::db::DbImpl;
use astu::db::DbTaskStatus;
use astu::resolve::Host;
use astu::resolve::Target;
use clap::Args;
use clap::ValueEnum;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::sync::Semaphore;
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

#[derive(Debug)]
struct SpoolCleanup(Option<PathBuf>);

impl Drop for SpoolCleanup {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = std::fs::remove_file(path);
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
        if let Some(confirm) = self.confirm {
            if confirm != target_count {
                bail!("--confirm expected {target_count}, got {confirm}");
            }
            return Ok(());
        }

        if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            bail!("--confirm={target_count} is required in non-interactive mode");
        }

        eprintln!("Plan affects {target_count} targets.");
        eprint!("Enter target count to confirm: ");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        let parsed = answer
            .trim()
            .parse::<usize>()
            .map_err(|_| anyhow!("invalid confirmation input"))?;
        if parsed != target_count {
            bail!("confirmation failed: expected {target_count}, got {parsed}");
        }
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

    #[allow(clippy::too_many_lines)]
    pub async fn run_tasks(
        &self,
        db: DbImpl,
        job_id: &str,
        specs: Vec<TaskSpec>,
        auth: &AuthArgs,
        pipe_stdin: Option<PathBuf>,
    ) -> Result<()> {
        let _spool_cleanup = SpoolCleanup(pipe_stdin.clone());
        let DbImpl::Duck(db) = db;

        let command = specs.first().map(|x| x.command.clone()).unwrap_or_default();
        db.create_job(
            job_id,
            &command,
            i64::try_from(self.concurrency).unwrap_or(i64::MAX),
            i64::try_from(specs.len()).unwrap_or(i64::MAX),
        )
        .await?;

        let total_tasks = specs.len();
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
        let progress_tty = std::io::IsTerminal::is_terminal(&std::io::stderr());
        let completed = Arc::new(AtomicUsize::new(0));
        if progress_tty {
            eprintln!("Progress: 0/{total_tasks}");
        }

        for spec in specs {
            let task_id = Uuid::now_v7().hyphenated().to_string();
            if cancel.load(Ordering::SeqCst) {
                db.create_task(&task_id, job_id, &spec.target.to_string(), &spec.command)
                    .await?;
                if let Some(param) = &spec.param {
                    db.append_task_var(&task_id, "{param}", param).await?;
                }
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
                if progress_tty {
                    let done = completed.fetch_add(1, Ordering::SeqCst) + 1;
                    eprint!("\rProgress: {done}/{total_tasks}");
                    if done == total_tasks {
                        eprintln!();
                    }
                }
                continue;
            }

            let permit = sem.clone().acquire_owned().await?;
            db.create_task(&task_id, job_id, &spec.target.to_string(), &spec.command)
                .await?;
            if let Some(param) = &spec.param {
                db.append_task_var(&task_id, "{param}", param).await?;
            }

            let db = db.clone();
            let client_factory = client_factory.clone();
            let auth = auth.clone();
            let pipe_stdin = pipe_stdin.clone();
            let completed = completed.clone();

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
                let stdin_input = pipe_stdin.map(ExecStdin::SpoolFile);

                let t_exec = Instant::now();
                let result = client.exec(&spec.command, stdin_input).await;
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

                if progress_tty {
                    let done = completed.fetch_add(1, Ordering::SeqCst) + 1;
                    eprint!("\rProgress: {done}/{total_tasks}");
                    if done == total_tasks {
                        eprintln!();
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
}

pub async fn read_stdin_all_if_piped() -> Result<Option<Vec<u8>>> {
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Ok(None);
    }

    let mut buf = Vec::new();
    tokio::io::stdin().read_to_end(&mut buf).await?;
    Ok(Some(buf))
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
    let host = target.host().map_or_else(
        || "{host}".to_owned(),
        |h| match h {
            Host::Ip(ip) => ip.to_string(),
            Host::Domain(domain) => domain,
        },
    );
    let user = target.user().unwrap_or("{user}").to_owned();
    let ip = target
        .ip()
        .map_or_else(|| "{ip}".to_owned(), |x| x.to_string());
    let param = param.unwrap_or("{param}").to_owned();

    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("host".to_owned(), host);
    vars.insert("user".to_owned(), user);
    vars.insert("ip".to_owned(), ip);
    vars.insert("param".to_owned(), param);

    strfmt::strfmt(template, &vars).unwrap_or_else(|_| template.to_owned())
}
