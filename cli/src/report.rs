use anyhow::Result;
use std::io::Write;
use tabled::settings::Style;
use tabled::Table;
use tabled::Tabled;

#[derive(Debug, Tabled)]
struct ErrorFreqRow {
    count: i64,
    value: String,
}

pub async fn print_error_freq_summary(db: &astu::db::DuckDb, job_id: &str) -> Result<()> {
    let rows = db.freq(astu::db::DbField::Error, job_id, None).await?;
    let view = rows
        .into_iter()
        .map(|row| ErrorFreqRow {
            count: row.count,
            value: row.value,
        })
        .collect::<Vec<_>>();

    println!("error-freq");
    if view.is_empty() {
        println!("(no rows)");
    } else {
        let mut table = Table::new(view);
        table.with(Style::markdown());
        println!("{table}");
    }
    let _ = std::io::stdout().flush();
    eprintln!("Use `astu output` or `astu freq` for result analysis");
    Ok(())
}
