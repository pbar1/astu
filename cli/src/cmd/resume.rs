use std::str::FromStr;

use anyhow::Result;
use astu::resolve::Target;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;
use crate::runtime::Runtime;

/// Resume a canceled job.
#[derive(Debug, Args)]
pub struct ResumeArgs {
    #[command(flatten)]
    job: crate::args::JobArgs,

    #[command(flatten)]
    auth_args: crate::args::AuthArgs,

    #[command(flatten)]
    action_args: crate::args::ActionArgs,
}

impl Run for ResumeArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let duck = runtime.db();
        let Some(job_id) = self.job.resolve(duck).await? else {
            return Ok(());
        };

        let canceled = duck.canceled_tasks_for_job(&job_id).await?;
        if canceled.is_empty() {
            println!("No canceled tasks to resume for job: {job_id}");
            return Ok(());
        }

        let command = duck.command_for_job(&job_id).await?.unwrap_or_default();
        println!("Resuming job: {job_id}");
        println!("Command: {command}");

        let specs = canceled
            .into_iter()
            .map(|(_task_id, target, task_command)| {
                let target = Target::from_str(&target)?;
                let effective_command = if task_command.is_empty() {
                    command.clone()
                } else {
                    task_command
                };
                Ok(crate::args::TaskSpec {
                    target,
                    command: effective_command,
                    param: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut by_command: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for spec in specs {
            by_command
                .entry(spec.command)
                .or_default()
                .push(spec.target.to_string());
        }

        let exe = std::env::current_exe()?;
        for (command, targets) in by_command {
            let mut child = tokio::process::Command::new(&exe);
            child
                .arg("--data-dir")
                .arg(runtime.data_dir().as_str())
                .arg("run");

            for target in &targets {
                child.arg("-T").arg(target);
            }

            child
                .arg(format!("--confirm={}", targets.len()))
                .arg("-u")
                .arg(&self.auth_args.user);

            if let Some(socket) = &self.auth_args.ssh_agent {
                child.arg("--ssh-agent").arg(socket);
            }
            if let Some(path) = &self.auth_args.password_file {
                child.arg("--password-file").arg(path);
            }
            if let Some(path) = &self.auth_args.ssh_key {
                child.arg("--ssh-key").arg(path);
            }

            child.arg(command);
            let status = child.status().await?;
            if !status.success() {
                anyhow::bail!("resume failed executing replacement run command");
            }
        }
        Ok(())
    }
}
