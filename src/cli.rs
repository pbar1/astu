mod cp;
mod exec;

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

use crate::target_types::InteractiveShell;

// Inspired by Rain's Rust CLI recommendations
// https://rust-cli-recommendations.sunshowers.io/handling-arguments.html
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(flatten)]
    global_args: GlobalArgs,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Args)]
struct GlobalArgs {
    /// Config file location. Defaults to `$XDG_CONFIG_HOME/kush/config.toml` if
    /// unset
    #[clap(long = "config")]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a command or shell on a target
    #[clap(hide = true)]
    Exec(self::exec::ExecArgs),

    /// Copy files and directories to and from targets
    #[clap(hide = true)]
    Cp(self::cp::CpArgs),
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn run(&self) -> Result<()> {
        // If no subcommand is passed, assume interactive k8s exec is desired
        // TODO: Maybe allow interactive selection of target types here
        let Some(command) = &self.command else {
            return crate::target_types::k8s::K8sExec::default().interactive_shell();
        };

        match command {
            Commands::Exec(args) => args.run(),
            Commands::Cp(args) => args.run(),
        }
    }
}

pub trait Runnable {
    // TODO: Consider what to do with GlobalArgs
    fn run(&self) -> Result<()>;
}
