use std::process::Stdio;

use anyhow::Result;

#[derive(Default)]
pub struct DockerExec {}

impl DockerExec {
    pub fn select_container() -> Result<String> {
        let output = std::process::Command::new("docker")
            .args(["ps", "--format={{ .ID }}"])
            .output()?;

        let containers = output.stdout;

        let selector = crate::selection::auto();
        let sel = selector.filter(containers, Some("docker inspect {}"))?;

        Ok(sel)
    }

    pub fn select_shell(container: &str) -> Result<String> {
        let output = std::process::Command::new("docker")
            .args(["exec", container, "cat", "/etc/shells"])
            .output()?;

        let shells = String::from_utf8(output.stdout)?
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes();

        let selector = crate::selection::auto();
        let sel = selector.filter(shells, None)?;

        Ok(sel)
    }

    pub fn exec_cmd(container: &str, cmd: &str) -> Result<()> {
        std::process::Command::new("docker")
            .args(["exec", "--interactive", "--tty", container, cmd])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        Ok(())
    }
}

impl super::InteractiveShell for DockerExec {
    // TODO: Support preferred shell via user preferences
    fn interactive_shell(&self) -> Result<()> {
        let container = Self::select_container()?;

        let shell = Self::select_shell(&container)?;

        Self::exec_cmd(&container, &shell)?;

        Ok(())
    }
}
