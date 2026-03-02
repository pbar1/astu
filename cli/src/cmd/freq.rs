use anyhow::Result;
use serde::Serialize;
use astu::util::id::Id;
use clap::Args;
use tabled::Tabled;

use crate::cmd::Run;
use crate::field::ResultFieldArg;
use crate::runtime::Runtime;

/// Aggregate results from a prior run into frequency tables.
#[derive(Debug, Args)]
pub struct FreqArgs {
    #[arg(value_enum)]
    fields: Vec<ResultFieldArg>,

    #[command(flatten)]
    job_args: crate::args::JobArgs,

    #[arg(long)]
    contains: Option<String>,
}

#[derive(Debug, Serialize, Tabled)]
struct FreqRowView {
    count: i64,
    value: String,
}

impl Run for FreqArgs {
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

        if matches!(runtime.output(), crate::args::OutputFormat::Json) {
            let mut out = serde_json::Map::new();
            for field in fields {
                let rows = runtime
                    .db()
                    .freq(field.into_db(), &job_id, self.contains.as_deref())
                    .await?;
                let view = rows
                    .into_iter()
                    .map(|row| FreqRowView {
                        count: row.count,
                        value: row.value,
                    })
                    .collect::<Vec<_>>();
                out.insert(field.freq_title().to_owned(), serde_json::to_value(view)?);
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
                .freq(field.into_db(), &job_id, self.contains.as_deref())
                .await?;
            let view = rows
                .into_iter()
                .map(|row| FreqRowView {
                    count: row.count,
                    value: row.value,
                })
                .collect::<Vec<_>>();
            rendered.push_str(&crate::cmd::render::section_table(field.freq_title(), view));
        }
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;

        Ok(())
    }
}
