use anyhow::Result;
use astu::util::id::Id;
use clap::Args;
use tokio::sync::watch;
use uuid::Uuid;

use crate::cmd::Run;
use crate::runtime::Runtime;

/// Execute commands on targets.
#[derive(Debug, Args)]
pub struct ExecArgs {
    #[command(flatten)]
    resolution_args: crate::args::ResolutionArgs,

    #[command(flatten)]
    auth_args: crate::args::AuthArgs,

    #[command(flatten)]
    action_args: crate::args::ActionArgs,

    /// Command template.
    command: String,
}

impl Run for ExecArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let has_stdin_target_file = self
            .resolution_args
            .target_files
            .iter()
            .any(|x| x == "-" || x == "/dev/stdin");

        let mode = self
            .action_args
            .infer_input_mode(&self.command, has_stdin_target_file);

        let prepared_stdin = if mode == crate::args::InputMode::Pipe {
            crate::args::PreparedStdin::default()
        } else {
            crate::args::read_stdin_for_mode(runtime.data_dir().as_str(), "", mode).await?
        };

        let stdin_targets_owned = if mode == crate::args::InputMode::Target {
            Some(String::from_utf8_lossy(&prepared_stdin.bytes).into_owned())
        } else {
            None
        };

        let targets = self
            .resolution_args
            .set_with_default(stdin_targets_owned.as_deref())
            .await?;
        let specs =
            crate::args::build_task_specs(targets, &self.command, mode, &prepared_stdin.bytes);
        let target_count = specs.len();
        self.action_args.require_confirm(target_count)?;

        let job_id = Uuid::now_v7().hyphenated().to_string();
        let mut pipe_pump = None;
        let (interrupt_tx, interrupt_rx) = watch::channel(false);
        let pipe_stdin = if mode == crate::args::InputMode::Pipe {
            let spool = crate::args::read_stdin_for_mode(runtime.data_dir().as_str(), &job_id, mode)
                .await?
                .spool;
            if let Some(spool_ref) = spool.clone() {
                let cancel_rx = interrupt_rx.clone();
                pipe_pump = Some(tokio::spawn(async move {
                    crate::action::stdin::pump_stdin_to_spool_with_cancel(&spool_ref, cancel_rx)
                        .await
                }));
            }
            spool
        } else {
            None
        };

        self.action_args
            .run_tasks(
                runtime.db().clone(),
                &job_id,
                specs,
                &self.auth_args,
                pipe_stdin,
                Some(interrupt_tx.clone()),
            )
            .await?;

        let _ = interrupt_tx.send(true);
        if let Some(handle) = pipe_pump {
            if !handle.is_finished() {
                handle.abort();
            }
            let _ = handle.await;
        }

        crate::report::print_error_freq_summary(runtime.db(), &job_id, runtime.output()).await?;

        Ok(())
    }
}
