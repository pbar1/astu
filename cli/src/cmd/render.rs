use anyhow::Context;
use anyhow::Result;
use std::io::IsTerminal;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use tabled::settings::Style;
use tabled::Table;
use tabled::Tabled;

pub fn modern_table<T: Tabled>(rows: Vec<T>) -> String {
    if rows.is_empty() {
        return "(no rows)\n".to_owned();
    }
    let mut table = Table::new(rows);
    table.with(Style::modern());
    format!("{table}\n")
}

pub fn section_table<T: Tabled>(title: &str, rows: Vec<T>) -> String {
    let mut out = String::new();
    out.push_str(title);
    out.push('\n');
    out.push_str(&modern_table(rows));
    out
}

pub fn emit_with_optional_pager(content: &str, enable_pager: bool) -> Result<()> {
    if !enable_pager || !should_use_pager() {
        return crate::ui::out(content);
    }

    let Ok(mut child) = Command::new("less")
        .args(["-FIRX"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    else {
        return crate::ui::out(content);
    };

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .context("write pager input")?;
    }

    let status = child.wait().context("wait pager")?;
    if !status.success() {
        anyhow::bail!("pager exited with status {status}");
    }
    Ok(())
}

fn should_use_pager() -> bool {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return false;
    }
    if matches!(std::env::var("ASTU_NO_PAGER").as_deref(), Ok("1")) {
        return false;
    }
    true
}
