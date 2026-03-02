use anyhow::Result;
use astu::db::DbField;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use clap::ValueEnum;
use tabled::Tabled;

use crate::cmd::Run;

/// Aggregate results from a prior run into frequency tables.
#[derive(Debug, Args)]
pub struct FreqArgs {
    #[arg(value_enum)]
    fields: Vec<FieldArg>,

    #[command(flatten)]
    job_args: crate::args::JobArgs,

    #[arg(long)]
    contains: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FieldArg {
    Stdout,
    Stderr,
    Exitcode,
    Error,
}

#[derive(Debug, Tabled)]
struct FreqRowView {
    count: i64,
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
            Self::Error => "error-freq",
        }
    }
}

impl Run for FreqArgs {
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

        let mut rendered = String::new();
        for (idx, field) in fields.into_iter().enumerate() {
            if idx > 0 {
                rendered.push('\n');
            }
            let rows = db
                .freq(field.into_db(), &job_id, self.contains.as_deref())
                .await?;
            let view = rows
                .into_iter()
                .map(|row| FreqRowView {
                    count: row.count,
                    value: row.value,
                })
                .collect::<Vec<_>>();
            rendered.push_str(&crate::cmd::render::section_table(field.as_str(), view));
        }
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;

        Ok(())
    }
}
