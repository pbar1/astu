mod exec;
mod ping;
mod resolve;

use anyhow::Result;
use astu_util::id::Id;
use astu_util::id::IdGenerator;
use astu_util::id::SonyflakeGenerator;
use clap::Parser;
use clap::Subcommand;
use enum_dispatch::enum_dispatch;

use crate::argetype::GlobalArgs;

/// Remote execution multitool
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[command(flatten)]
    global_args: GlobalArgs,
}

#[derive(Debug, Subcommand)]
#[enum_dispatch]
enum Command {
    Resolve(resolve::ResolveArgs),
    Ping(ping::PingArgs),
    Exec(exec::ExecArgs),
}

/// Subcommands should implement [`Run`] to be executed at runtime.
#[enum_dispatch]
pub trait Run {
    async fn run(&self, id: Id) -> Result<()>;
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.global_args.init_tracing()?;

    let id = SonyflakeGenerator::from_hostname()?.id_now();

    match cli.command {
        Command::Exec(args) => args.run(id).await,
        Command::Resolve(args) => args.run(id).await,
        Command::Ping(args) => args.run(id).await,
    }
}
