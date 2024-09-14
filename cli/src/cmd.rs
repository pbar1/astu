mod exec;
mod ping;
mod resolve;

use astu_util::id::Id;
use astu_util::id::IdGenerator;
use astu_util::id::SonyflakeGenerator;
use clap::Parser;
use clap::Subcommand;

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
enum Command {
    Resolve(resolve::ResolveArgs),
    Ping(ping::PingArgs),
    Exec(exec::ExecArgs),
}

#[async_trait::async_trait]
pub trait Run {
    async fn run(&self, id: Id) -> anyhow::Result<()>;
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.global_args.init_tracing()?;

    let id = SonyflakeGenerator::from_hostname()?.id_now();

    let command: Box<dyn Run> = match cli.command {
        Command::Exec(args) => Box::new(args),
        Command::Resolve(args) => Box::new(args),
        Command::Ping(args) => Box::new(args),
    };
    command.run(id).await?;

    Ok(())
}
