use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use uuid::Uuid;

use crate::cmd::Run;

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
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let stdin_bytes = crate::args::read_stdin_all_if_piped()
            .await?
            .unwrap_or_default();
        let has_stdin_target_file = self
            .resolution_args
            .target_files
            .iter()
            .any(|x| x == "-" || x == "/dev/stdin");

        let mode = self
            .action_args
            .infer_input_mode(&self.command, has_stdin_target_file);

        let stdin_str = String::from_utf8_lossy(&stdin_bytes);
        let stdin_targets = if mode == crate::args::InputMode::Target {
            Some(stdin_str.as_ref())
        } else {
            None
        };

        let targets = self.resolution_args.set_with_default(stdin_targets).await?;
        let specs = crate::args::build_task_specs(targets, &self.command, mode, &stdin_bytes);
        let target_count = specs.len();
        self.action_args.require_confirm(target_count)?;

        let job_id = Uuid::now_v7().hyphenated().to_string();

        let data_dir = std::env::var("ASTU_DATA_DIR").unwrap_or_else(|_| {
            astu::util::dirs::data_dir("astu")
                .to_string_lossy()
                .to_string()
        });
        let spool =
            crate::args::ActionArgs::maybe_spool_stdin(&data_dir, &job_id, mode, &stdin_bytes)?;

        self.action_args
            .run_tasks(db.clone(), &job_id, specs, &self.auth_args, spool)
            .await?;

        let DbImpl::Duck(db) = db;
        crate::report::print_error_freq_summary(&db, &job_id).await?;

        Ok(())
    }
}
