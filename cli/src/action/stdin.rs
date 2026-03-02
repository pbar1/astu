use anyhow::Result;
use astu::resolve::Target;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

use crate::args::InputMode;
use crate::args::TaskSpec;

pub async fn read_stdin_all_if_piped() -> Result<Option<Vec<u8>>> {
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Ok(None);
    }

    let mut buf = Vec::new();
    tokio::io::stdin().read_to_end(&mut buf).await?;
    Ok(Some(buf))
}

pub fn maybe_spool_stdin(
    data_dir: &str,
    job_id: &str,
    mode: InputMode,
    stdin: &[u8],
) -> Result<Option<PathBuf>> {
    if mode != InputMode::Pipe || stdin.is_empty() {
        return Ok(None);
    }

    let spool_dir = PathBuf::from(data_dir).join("spool");
    std::fs::create_dir_all(&spool_dir)?;
    let path = spool_dir.join(format!("{job_id}.stdin"));
    std::fs::write(&path, stdin)?;
    Ok(Some(path))
}

pub fn build_task_specs(
    targets: Vec<Target>,
    command: &str,
    mode: InputMode,
    stdin: &[u8],
) -> Vec<TaskSpec> {
    match mode {
        InputMode::Param => {
            let params = String::from_utf8_lossy(stdin)
                .split_whitespace()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();
            if params.is_empty() {
                return targets
                    .into_iter()
                    .map(|t| TaskSpec {
                        target: t,
                        command: command.to_owned(),
                        param: None,
                    })
                    .collect();
            }

            let mut out = Vec::new();
            for target in targets {
                for param in &params {
                    out.push(TaskSpec {
                        target: target.clone(),
                        command: crate::action::template::render_command(
                            command,
                            &target,
                            Some(param),
                        ),
                        param: Some(param.clone()),
                    });
                }
            }
            out
        }
        _ => targets
            .into_iter()
            .map(|target| TaskSpec {
                command: crate::action::template::render_command(command, &target, None),
                target,
                param: None,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use astu::resolve::Target;

    use super::build_task_specs;
    use crate::args::InputMode;

    #[test]
    fn param_mode_expands_per_target_and_param() {
        let targets = vec![
            Target::from_str("dummy://a").expect("target"),
            Target::from_str("dummy://b").expect("target"),
        ];
        let specs = build_task_specs(targets, "echo {param}", InputMode::Param, b"x y");
        assert_eq!(specs.len(), 4);
    }
}
