use std::ffi::OsStr;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;

async fn run_command<S, I>(program: S, args: I) -> Result<()>
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
{
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id().context("Failed to get PID")?;

    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    let mut stdout_done = false;
    let mut stderr_done = false;

    while !stdout_done || !stderr_done {
        tokio::select! {
            line = stdout_lines.next_line(), if !stdout_done => {
                match line? {
                    Some(text) => println!("stdout[{pid}]: {text}"),
                    None => stdout_done = true,
                }
            }
            line = stderr_lines.next_line(), if !stderr_done => {
                match line? {
                    Some(text) => eprintln!("stderr[{pid}]: {text}"),
                    None => stderr_done = true,
                }
            }
        }
    }

    let status = child.wait().await?;
    println!("status[{pid}]: {status}");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd1 = tokio::spawn(run_command("seq", ["1", "10"]));
    let cmd2 = tokio::spawn(run_command("seq", ["1", "10"]));
    let cmd3 = tokio::spawn(run_command("seq", ["1", "10"]));

    tokio::join!(cmd1, cmd2, cmd3);

    Ok(())
}
