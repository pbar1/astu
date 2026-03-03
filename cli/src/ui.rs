use anyhow::Context;
use anyhow::Result;
use console::Term;

pub fn out(content: &str) -> Result<()> {
    Term::stdout()
        .write_str(content)
        .context("write stdout content")
}

pub fn out_line(content: &str) -> Result<()> {
    Term::stdout()
        .write_line(content)
        .context("write stdout line")
}

pub fn err_line(content: &str) -> Result<()> {
    Term::stderr()
        .write_line(content)
        .context("write stderr line")
}
