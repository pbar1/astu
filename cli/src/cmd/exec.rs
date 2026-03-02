use std::str::FromStr;

use anyhow::Result;
use astu::db::DbField;
use astu::db::DbImpl;
use astu::resolve::Target;
use astu::util::id::Id;
use clap::Args;
use uuid::Uuid;

use crate::cmd::Run;
use crate::cmd::common;

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
        let stdin_bytes = common::read_stdin_all_if_piped().await?.unwrap_or_default();
        let has_stdin_target_file = self
            .resolution_args
            .target_files
            .iter()
            .any(|x| x == "-" || x == "/dev/stdin");

        let mode = common::infer_input_mode(&self.action_args, &self.command, has_stdin_target_file);

        let stdin_str = String::from_utf8_lossy(&stdin_bytes);
        let stdin_targets = if mode == common::InputMode::Target {
            Some(stdin_str.as_ref())
        } else {
            None
        };

        let mut set = self.resolution_args.set_with_stdin(stdin_targets).await?;
        if set.is_empty() {
            set.insert(Target::from_str("local:")?);
        }
        let targets = common::normalize_targets(set);

        let specs = common::build_task_specs(targets, &self.command, mode, &stdin_bytes);
        let target_count = specs.len();
        common::require_confirm(self.action_args.confirm, target_count)?;

        let job_id = Uuid::now_v7().hyphenated().to_string();

        let data_dir = std::env::var("ASTU_DATA_DIR").unwrap_or_else(|_| {
            std::env::temp_dir()
                .join("astu")
                .to_string_lossy()
                .to_string()
        });
        let spool = common::maybe_spool_stdin(&data_dir, &job_id, mode, &stdin_bytes)?;

        common::run_tasks(
            db.clone(),
            &job_id,
            specs,
            &self.auth_args,
            &self.action_args,
            spool,
        )
        .await?;

        let DbImpl::Duck(db) = db;
        let rows = db.freq(DbField::Error, &job_id, None).await?;
        if rows.is_empty() {
            println!("error-freq\n(no rows)");
        } else {
            println!("error-freq");
            for row in rows {
                println!("{}\t{}", row.count, row.value);
            }
        }
        eprintln!("Use `astu output` or `astu freq` for result analysis");

        Ok(())
    }
}
