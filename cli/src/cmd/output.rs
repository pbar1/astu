use anyhow::Result;
use astu::db::DbField;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use clap::ValueEnum;
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

        for field in fields {
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
                    task_id: row.task_id,
                    target: row.target,
                    value: row.value,
                })
                .collect::<Vec<_>>();
            crate::cmd::render::print_section_table(field.as_str(), view);
        }

        Ok(())
    }
}
