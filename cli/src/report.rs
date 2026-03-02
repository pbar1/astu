use anyhow::Result;
use serde::Serialize;
use std::io::Write;
use tabled::settings::Style;
use tabled::Table;
use tabled::Tabled;

#[derive(Debug, Serialize, Tabled)]
struct ErrorFreqRow {
    count: i64,
    value: String,
}

pub async fn print_error_freq_summary(
    db: &astu::db::DuckDb,
    job_id: &str,
    output: crate::args::OutputFormat,
) -> Result<()> {
    let rows = db.freq(astu::db::DbField::Error, job_id, None).await?;
    let view = rows
        .into_iter()
        .map(|row| ErrorFreqRow {
            count: row.count,
            value: row.value,
        })
        .collect::<Vec<_>>();

    if matches!(output, crate::args::OutputFormat::Json) {
        let value = serde_json::json!({ "error-freq": view });
        let rendered = format!("{}\n", serde_json::to_string_pretty(&value)?);
        crate::cmd::render::emit_with_optional_pager(&rendered, true)?;
        return Ok(());
    }

    println!("error-freq");
    if view.is_empty() {
        println!("(no rows)");
    } else {
        let mut table = Table::new(view);
        table.with(Style::modern());
        println!("{table}");
    }
    let _ = std::io::stdout().flush();
    eprintln!();
    eprintln!("Use `astu output` or `astu freq` for result analysis");
    Ok(())
}
