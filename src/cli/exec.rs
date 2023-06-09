use anyhow::Result;
use clap::Args;
use url::Url;

use super::Runnable;

#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Target for execution
    target: Option<Url>,

    /// Command to execute on the target. If omitted, will attempt to spawn an
    /// interactive shell
    #[clap(trailing_var_arg = true)]
    command: Vec<String>,
}

impl Runnable for ExecArgs {
    fn run(&self) -> Result<()> {
        dbg!(&self);
        todo!()
    }
}
