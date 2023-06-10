use anyhow::Result;
use clap::Args;

use super::Runnable;

#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Target for execution
    target: Option<String>,

    /// Command to execute on the target. If omitted, will attempt to spawn an
    /// interactive shell
    #[clap(trailing_var_arg = true)]
    command: Vec<String>,
}

impl Runnable for ExecArgs {
    fn run(&self) -> Result<()> {
        // TODO wip
        let target = self.target.clone().unwrap_or("".to_owned());
        crate::target_types::dispatch_interactive_shell(&target)
    }
}
