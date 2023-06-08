use std::process::Stdio;

use anyhow::Result;

use crate::selection::preview_select;

pub fn select_pod() -> Result<String> {
    let output = std::process::Command::new("kubectl")
        .args(["get", "pods", "--output=name"])
        .output()?;

    let pods = String::from_utf8(output.stdout)?.replace("pod/", "");

    let sel = preview_select(pods, Some("kubectl describe pod {}"))?;

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

    let containers = String::from_utf8(output.stdout)?;

    let sel = preview_select(
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

    let shells: String = String::from_utf8(output.stdout)?
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    let sel = preview_select(shells, None)?;

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

// TODO: Support default container annotation
// TODO: Support preferred shell via user preferences
pub fn exec_shell() -> Result<()> {
    let pod = select_pod()?;

    let container = select_container(&pod)?;

    let shell = select_shell(&pod, &container)?;

    exec_cmd(&pod, &container, &shell)?;

    Ok(())
}
