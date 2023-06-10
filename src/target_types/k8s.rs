use std::process::Stdio;

use anyhow::Result;

#[derive(Default)]
pub struct K8sExec {}

impl K8sExec {
    pub fn select_pod() -> Result<String> {
        let output = std::process::Command::new("kubectl")
            .args(["get", "pods", "--output=name"])
            .output()?;

        let pods = String::from_utf8(output.stdout)?
            .replace("pod/", "")
            .into_bytes();

        let selector = crate::selection::auto();
        let sel = selector.filter(pods, Some("kubectl describe pod {}"))?;

        Ok(sel)
    }

    pub fn select_container(pod: &str) -> Result<String> {
        let output = std::process::Command::new("kubectl")
            .args([
                "get",
                "pod",
                pod,
                r#"--output=jsonpath={range .spec.containers[*]}{.name}{"\n"}"#,
            ])
            .output()?;

        let containers = output.stdout;

        let selector = crate::selection::auto();
        let sel = selector.filter(
        containers,
        Some(&format!(
            "kubectl get pod {pod} --output=jsonpath='{{.spec.containers[?(@.name==\"{{}}\")]}}' | jq"
        )),
    )?;

        Ok(sel)
    }

    pub fn select_shell(pod: &str, container: &str) -> Result<String> {
        let output = std::process::Command::new("kubectl")
            .args([
                "exec",
                pod,
                &format!("--container={container}"),
                "--",
                "cat",
                "/etc/shells",
            ])
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

    pub fn exec_cmd(pod: &str, container: &str, cmd: &str) -> Result<()> {
        std::process::Command::new("kubectl")
            .args([
                "exec",
                pod,
                &format!("--container={container}"),
                "--stdin",
                "--tty",
                "--",
                cmd,
            ])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        Ok(())
    }
}

impl super::InteractiveShell for K8sExec {
    // TODO: Support default container annotation
    // TODO: Support preferred shell via user preferences
    fn interactive_shell(&self) -> Result<()> {
        let pod = Self::select_pod()?;

        let container = Self::select_container(&pod)?;

        let shell = Self::select_shell(&pod, &container)?;

        Self::exec_cmd(&pod, &container, &shell)?;

        Ok(())
    }
}
