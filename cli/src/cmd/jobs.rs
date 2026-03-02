use anyhow::Result;
use serde::Serialize;
use astu::util::id::Id;
use clap::Args;
use tabled::Tabled;

use crate::cmd::Run;
use crate::runtime::Runtime;

/// Display jobs metadata.
#[derive(Debug, Args)]
pub struct JobsArgs {
    #[arg(long, default_value_t = 50)]
    limit: i64,
}

#[derive(Debug, Serialize, Tabled)]
struct JobRowView {
    job_id: String,
    started_at: String,
    task_count: i64,
    command: String,
}

impl Run for JobsArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let rows = runtime.db().jobs(self.limit).await?;
        let view = rows
            .into_iter()
            .map(|row| JobRowView {
                job_id: row.job_id,
                started_at: row.started_at,
                task_count: row.task_count,
                command: row.command,
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
