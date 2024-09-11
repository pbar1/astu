#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod exec;
mod ping;
mod resolve;

use clap::Parser;
use clap::Subcommand;

/// Hello friend.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Exec(exec::ExecArgs),
    Resolve(resolve::ResolveArgs),
    Ping(ping::PingArgs),
}

#[async_trait::async_trait]
pub trait Run {
    async fn run(&self) -> anyhow::Result<()>;
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let command: Box<dyn Run> = match cli.command {
        Command::Exec(args) => Box::new(args),
        Command::Resolve(args) => Box::new(args),
        Command::Ping(args) => Box::new(args),
    };
    command.run().await?;

    Ok(())
}
