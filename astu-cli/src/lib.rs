mod arg;
mod cmd;
mod run;

use clap::Parser;

pub use crate::cmd::Command;
pub use crate::run::Run;

#[allow(clippy::unused_async)]
pub async fn run() -> eyre::Result<()> {
    let cli = Cli::parse();
    cli.command.run().await
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub global: arg::GlobalFlags,

    #[command(subcommand)]
    pub command: Command,
}
