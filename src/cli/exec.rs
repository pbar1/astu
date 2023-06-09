use clap::Args;
use url::Url;

#[derive(Debug, Args)]
pub(crate) struct ExecArgs {
    /// Target for execution
    target: Option<Url>,

    /// Command to execute on the target. If omitted, will attempt to spawn an
    /// interactive shell
    #[clap(trailing_var_arg = true)]
    command: Vec<String>,
}
