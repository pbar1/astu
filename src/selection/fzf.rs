use std::io::Write;
use std::process::Stdio;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;

use super::Selector;

#[derive(Default)]
pub(crate) struct FzfSelector {}

impl Selector for FzfSelector {
    fn filter(&self, source: Vec<u8>, preview: Option<&str>) -> Result<String> {
        let mut cmd = std::process::Command::new("fzf");

        if let Some(preview) = preview {
            cmd.arg(&format!("--preview={preview}"));
        }

        // Stdout needs to be piped so `wait_with_output()` below can capture it, while
        // stderr needs to remain inherited from the parent so it can be printed
        // to the terminal for user interaction. Stdin needs to be piped both so it can
        // receive input below, and because if it's inherited FZF will throw the
        // error `Failed to read /dev/tty`.
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let child = cmd.spawn()?;

        // TODO: Sometimes leading whitespace appears in the filter input
        // Write the contents of `source` to FZF stdin where it will be filtered
        // interactively by the user
        child
            .stdin
            .as_ref()
            .context("Unable to open FZF child stdin")?
            .write_all(source.as_ref())
            .context("Unable to write to FZF child stdin")?;

        let output = child.wait_with_output()?;

        // Ideally we'd capture the contents of stderr if the child process was not
        // successful, but because its inherited for interactive purposes, it
        // would just be empty here
        if !output.status.success() {
            bail!("FZF selection failed");
        }

        let sel = String::from_utf8(output.stdout)?.trim().to_owned();

        Ok(sel)
    }
}
