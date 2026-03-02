use anyhow::Result;
use astu::db::DbField;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use clap::ValueEnum;
use std::collections::HashMap;
use tabled::Tabled;

use crate::cmd::Run;

/// Replay outputs from a prior run.
#[derive(Debug, Args)]
pub struct OutputArgs {
    #[arg(value_enum)]
    fields: Vec<FieldArg>,

    #[command(flatten)]
    job_args: crate::args::JobArgs,

    #[arg(long)]
    contains: Option<String>,

    #[arg(short = 'T', long = "target")]
    target: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FieldArg {
    Stdout,
    Stderr,
    Exitcode,
    Error,
}

#[derive(Debug, Tabled)]
struct OutputRowView {
    task_id: String,
    target: String,
    value: String,
}

impl FieldArg {
    const fn into_db(self) -> DbField {
        match self {
            Self::Stdout => DbField::Stdout,
            Self::Stderr => DbField::Stderr,
            Self::Exitcode => DbField::Exitcode,
            Self::Error => DbField::Error,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::Exitcode => "exitcode",
            Self::Error => "error",
        }
    }
}

impl Run for OutputArgs {
    async fn run(&self, _id: Id, db: DbImpl) -> Result<()> {
        let DbImpl::Duck(db) = db;
        let Some(job_id) = self.job_args.resolve(&db).await? else {
            return Ok(());
        };

        let fields = if self.fields.is_empty() {
            vec![
                FieldArg::Stdout,
                FieldArg::Stderr,
                FieldArg::Exitcode,
                FieldArg::Error,
            ]
        } else {
            self.fields.clone()
        };

        let needs_vars = fields
            .iter()
            .any(|field| matches!(field, FieldArg::Stdout | FieldArg::Stderr));
        let vars_by_task = if needs_vars {
            db.task_vars_for_job(&job_id).await?
        } else {
            HashMap::new()
        };

        let mut rendered = String::new();
        for (idx, field) in fields.into_iter().enumerate() {
            if idx > 0 {
                rendered.push('\n');
            }
            let rows = db
                .output(
                    field.into_db(),
                    &job_id,
                    self.contains.as_deref(),
                    self.target.as_deref(),
                )
                .await?;
            let view = rows
                .into_iter()
                .map(|row| OutputRowView {
                    task_id: row.task_id.clone(),
                    target: row.target,
                    value: denormalize_value(&row.value, vars_by_task.get(&row.task_id)),
                })
                .collect::<Vec<_>>();
            rendered.push_str(&crate::cmd::render::section_table(field.as_str(), view));
        }
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;

        Ok(())
    }
}

fn denormalize_value(value: &str, vars: Option<&Vec<(String, String)>>) -> String {
    let mut out = value.to_owned();
    if let Some(vars) = vars {
        for (token, raw) in vars {
            out = out.replace(token, raw);
        }
    }
    out
}
