use anyhow::Result;
use astu::db::DbField;
use astu::db::DbImpl;
use astu::util::id::Id;
use clap::Args;
use clap::ValueEnum;

use crate::cmd::Run;

/// Replay outputs from a prior run.
#[derive(Debug, Args)]
pub struct OutputArgs {
    #[arg(value_enum)]
    fields: Vec<FieldArg>,

    #[arg(short = 'j', long)]
    job: Option<String>,

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
        let job_id = match &self.job {
            Some(job) => job.clone(),
            None => {
                let Some(job) = db.last_job_id().await? else {
                    return Ok(());
                };
                job
            }
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
            println!("{}", field.as_str());
            let rows = db
                .output(
                    field.into_db(),
                    &job_id,
                    self.contains.as_deref(),
                    self.target.as_deref(),
                )
                .await?;
            if rows.is_empty() {
                println!("(no rows)");
                continue;
            }
            for row in rows {
                println!("{}\t{}\t{}", row.task_id, row.target, row.value);
            }
        }

        Ok(())
    }
}
