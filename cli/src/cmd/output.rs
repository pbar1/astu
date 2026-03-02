use anyhow::Result;
use serde::Serialize;
use astu::util::id::Id;
use clap::Args;
use tabled::Tabled;

use crate::cmd::Run;
use crate::field::ResultFieldArg;
use crate::runtime::Runtime;

/// Replay outputs from a prior run.
#[derive(Debug, Args)]
pub struct OutputArgs {
    #[arg(value_enum)]
    fields: Vec<ResultFieldArg>,

    #[command(flatten)]
    job_args: crate::args::JobArgs,

    #[arg(long)]
    contains: Option<String>,

    #[arg(short = 'T', long = "target")]
    target: Option<String>,
}

#[derive(Debug, Serialize, Tabled)]
struct OutputRowView {
    task_id: String,
    target: String,
    value: String,
}

impl Run for OutputArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let Some(job_id) = self.job_args.resolve(runtime.db()).await? else {
            return Ok(());
        };

        let fields = if self.fields.is_empty() {
            vec![
                ResultFieldArg::Stdout,
                ResultFieldArg::Stderr,
                ResultFieldArg::Exitcode,
                ResultFieldArg::Error,
            ]
        } else {
            self.fields.clone()
        };

        let needs_vars = fields
            .iter()
            .any(|field| matches!(field, ResultFieldArg::Stdout | ResultFieldArg::Stderr));

        if matches!(runtime.output(), crate::args::OutputFormat::Json) {
            let mut out = serde_json::Map::new();
            for field in fields {
                let rows = runtime
                    .db()
                    .output(
                        field.into_db(),
                        &job_id,
                        self.contains.as_deref(),
                        self.target.as_deref(),
                    )
                    .await?;
                let mut view = Vec::with_capacity(rows.len());
                for row in rows {
                    let vars_for_task = if needs_vars {
                        runtime.db().task_vars_for_task(&row.task_id).await?
                    } else {
                        Vec::new()
                    };
                    view.push(OutputRowView {
                        task_id: row.task_id.clone(),
                        target: row.target,
                        value: denormalize_value(&row.value, Some(&vars_for_task)),
                    });
                }
                out.insert(field.output_title().to_owned(), serde_json::to_value(view)?);
            }
            let rendered = format!("{}\n", serde_json::to_string_pretty(&out)?);
            crate::cmd::render::emit_with_optional_pager(&rendered, true)?;
            return Ok(());
        }

        let mut rendered = String::new();
        for (idx, field) in fields.into_iter().enumerate() {
            if idx > 0 {
                rendered.push('\n');
            }
            let rows = runtime
                .db()
                .output(
                    field.into_db(),
                    &job_id,
                    self.contains.as_deref(),
                    self.target.as_deref(),
                )
                .await?;
            let mut view = Vec::with_capacity(rows.len());
            for row in rows {
                let vars_for_task = if needs_vars {
                    runtime.db().task_vars_for_task(&row.task_id).await?
                } else {
                    Vec::new()
                };
                view.push(OutputRowView {
                    task_id: row.task_id.clone(),
                    target: row.target,
                    value: denormalize_value(&row.value, Some(&vars_for_task)),
                });
            }
            rendered.push_str(&crate::cmd::render::section_table(
                field.output_title(),
                view,
            ));
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
