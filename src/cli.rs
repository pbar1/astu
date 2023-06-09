mod cp;
mod exec;

use anyhow::Result;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

use self::cp::CpArgs;
use self::exec::ExecArgs;

// Inspired by Rain's Rust CLI recommendations
// https://rust-cli-recommendations.sunshowers.io/handling-arguments.html
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub(crate) struct Cli {
    #[clap(flatten)]
    global_args: GlobalArgs,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
struct GlobalArgs {}

#[derive(Subcommand)]
enum Commands {
    /// Execute a command or shell on a target
    Exec(ExecArgs),

    /// Copy files and directories to and from targets
    #[clap(hide = true)]
    Cp(CpArgs),
}

impl Cli {
    pub(crate) fn new() -> Self {
        Self::parse()
    }

    pub(crate) fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Exec(_args) => {
                // FIXME: Generalize to more than just K8s
                crate::target_types::k8s::exec_shell()?;
            }
            Commands::Cp(args) => {
                dbg!(args);
                todo!();
            }
        }

        Ok(())
    }
}
