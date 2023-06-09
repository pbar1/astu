pub(crate) mod exec;

use anyhow::Result;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

use self::exec::ExecArgs;

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
pub struct GlobalArgs {}

#[derive(Subcommand)]
enum Commands {
    /// Execute a command or shell on a target
    Exec(ExecArgs),

    /// Copy files and directories to and from targets
    #[clap(hide = true)]
    Cp { target: Option<String> },

    /// Display what `kush` is capable of on the current system
    #[clap(hide = true)]
    Doctor,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Exec(args) => {
                println!("{args:?}");

                // FIXME: Generalize to more than just K8s
                crate::target_types::k8s::exec_shell()?;
            }
            Commands::Cp { target } => {
                println!("target: {target:?}");

                todo!();
            }
            Commands::Doctor => {
                println!("doctor!");

                todo!();
            }
        }

        Ok(())
    }
}
