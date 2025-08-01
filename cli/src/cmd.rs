mod exec;
mod ping;
mod resolve;

use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use astu::util::id::IdGenerator;
use astu::util::id::SonyflakeGenerator;
use clap::Parser;
use clap::Subcommand;
use enum_dispatch::enum_dispatch;

use crate::args::GlobalArgs;

/// Arbitrary Shell Targeting Utility
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(flatten)]
    global_args: GlobalArgs,
}

/// Subcommands must implement [`Run`] to be executed at runtime.
#[enum_dispatch]
pub trait Run {
    async fn run(&self, id: Id, db: DbImpl) -> Result<()>;
}

#[enum_dispatch(Run)]
#[derive(Debug, Subcommand)]
enum Command {
    Resolve(resolve::ResolveArgs),
    Ping(ping::PingArgs),
    Exec(exec::ExecArgs),
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.global_args.init_tracing()?;

    let id = SonyflakeGenerator::from_hostname()?.id_now();
    let db = cli.global_args.get_db().await?;

    cli.command.run(id, db).await
}
