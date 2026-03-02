use anyhow::Result;
use astu::util::id::Id;
use clap::Args;
use tabled::Tabled;

use crate::cmd::Run;
use crate::runtime::Runtime;

/// Display task trace timings and errors.
#[derive(Debug, Args)]
pub struct TraceArgs {
    #[command(flatten)]
    job_args: crate::args::JobArgs,

    #[arg(short = 'T', long = "target")]
    target: Option<String>,
}

#[derive(Debug, Tabled)]
struct TraceRowView {
    task_id: String,
    target: String,
    status: String,
    error: String,
    connect_ms: i64,
    auth_ms: i64,
    exec_ms: i64,
}

impl Run for TraceArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let Some(job_id) = self.job_args.resolve(runtime.db()).await? else {
            return Ok(());
        };

        let rows = runtime.db().trace(&job_id, self.target.as_deref()).await?;
        let view = rows
            .into_iter()
            .map(|row| TraceRowView {
                task_id: row.task_id,
                target: row.target,
                status: row.status,
                error: row.error,
                connect_ms: row.connect_ms,
                auth_ms: row.auth_ms,
                exec_ms: row.exec_ms,
            })
            .collect::<Vec<_>>();
        let rendered = crate::cmd::render::modern_table(view);
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;
        Ok(())
    }
}
