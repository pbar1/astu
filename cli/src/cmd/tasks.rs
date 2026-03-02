use anyhow::Result;
use serde::Serialize;
use astu::util::id::Id;
use clap::Args;
use tabled::Tabled;

use crate::cmd::Run;
use crate::runtime::Runtime;

/// Display tasks metadata for a job.
#[derive(Debug, Args)]
pub struct TasksArgs {
    #[command(flatten)]
    job_args: crate::args::JobArgs,
}

#[derive(Debug, Serialize, Tabled)]
struct TaskRowView {
    task_id: String,
    target: String,
    status: String,
    command: String,
    exit_code: String,
}

impl Run for TasksArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let Some(job_id) = self.job_args.resolve(runtime.db()).await? else {
            return Ok(());
        };

        let rows = runtime.db().tasks(&job_id).await?;
        let view = rows
            .into_iter()
            .map(|row| TaskRowView {
                task_id: row.task_id,
                target: row.target,
                status: row.status,
                command: row.command,
                exit_code: row.exit_code.map_or_else(String::new, |x| x.to_string()),
            })
            .collect::<Vec<_>>();
        if matches!(runtime.output(), crate::args::OutputFormat::Json) {
            println!("{}", serde_json::to_string_pretty(&view)?);
            return Ok(());
        }
        let rendered = crate::cmd::render::modern_table(view);
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;
        Ok(())
    }
}
