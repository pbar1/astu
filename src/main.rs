#![warn(clippy::pedantic)]

mod k8s;
mod selection;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::k8s::exec_shell;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a command in a target
    Exec { target: Option<String> },
    /// Copy files and directories to and from targets
    Cp { target: Option<String> },
    /// Display what `kush` is capable of on the current system
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Exec { target: _ } => {
            // FIXME: Generalize to more than just K8s
            exec_shell()?;
        }
        Commands::Cp { target } => {
            println!("target: {target:?}");
        }
        Commands::Doctor => {
            println!("doctor!");
        }
    }

    Ok(())
}
