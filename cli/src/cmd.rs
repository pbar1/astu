mod exec;
mod freq;
mod gc;
mod jobs;
mod output;
mod ping;
mod resume;
mod resolve;
mod tasks;
mod trace;

use anyhow::Result;
use astu::db::DbImpl;
use astu::util::id::Id;
use astu::util::id::IdGenerator;
use astu::util::id::SonyflakeGenerator;
use clap::Parser;
use clap::Subcommand;
use enum_dispatch::enum_dispatch;

use crate::args::GlobalArgs;

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
    #[command(visible_aliases = ["r", "exec"])]
    Run(exec::ExecArgs),
    #[command(visible_alias = "p")]
    Ping(ping::PingArgs),
    #[command(visible_aliases = ["l", "resolve"])]
    Lookup(resolve::ResolveArgs),
    Resume(resume::ResumeArgs),
    #[command(visible_alias = "f")]
    Freq(freq::FreqArgs),
    #[command(visible_aliases = ["o", "out"])]
    Output(output::OutputArgs),
    Trace(trace::TraceArgs),
    #[command(visible_aliases = ["j", "job"])]
    Jobs(jobs::JobsArgs),
    #[command(visible_aliases = ["t", "task"])]
    Tasks(tasks::TasksArgs),
    Gc(gc::GcArgs),
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _guards = cli.global_args.init_tracing()?;

    let id = SonyflakeGenerator::from_hostname()?.id_now();
    let db = cli.global_args.get_db().await?;

    cli.command.run(id, db).await
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Cli;

    #[test]
    fn parses_run_command() {
        let cli = Cli::try_parse_from(["astu", "run", "echo hi"]).expect("parse");
        let command = format!("{:?}", cli.command);
        assert!(command.contains("Run"), "{command}");
    }

    #[test]
    fn parses_exec_alias_for_run() {
        let cli = Cli::try_parse_from(["astu", "exec", "echo hi"]).expect("parse");
        let command = format!("{:?}", cli.command);
        assert!(command.contains("Run"), "{command}");
    }

    #[test]
    fn parses_lookup_aliases() {
        let cli = Cli::try_parse_from(["astu", "lookup"]).expect("parse");
        let command = format!("{:?}", cli.command);
        assert!(command.contains("Lookup"), "{command}");

        let cli = Cli::try_parse_from(["astu", "resolve"]).expect("parse");
        let command = format!("{:?}", cli.command);
        assert!(command.contains("Lookup"), "{command}");
    }

    #[test]
    fn parses_other_group_commands() {
        let cli = Cli::try_parse_from(["astu", "jobs"]).expect("parse");
        assert!(format!("{:?}", cli.command).contains("Jobs"));

        let cli = Cli::try_parse_from(["astu", "job"]).expect("parse");
        assert!(format!("{:?}", cli.command).contains("Jobs"));

        let cli = Cli::try_parse_from(["astu", "tasks"]).expect("parse");
        assert!(format!("{:?}", cli.command).contains("Tasks"));

        let cli = Cli::try_parse_from(["astu", "task"]).expect("parse");
        assert!(format!("{:?}", cli.command).contains("Tasks"));

        let cli = Cli::try_parse_from(["astu", "gc"]).expect("parse");
        assert!(format!("{:?}", cli.command).contains("Gc"));
    }

    #[test]
    fn rejects_removed_cp_command() {
        let error = Cli::try_parse_from(["astu", "cp"]).expect_err("must fail");
        let msg = error.to_string();
        assert!(msg.contains("unrecognized subcommand"), "{msg}");
    }

    #[test]
    fn parses_global_output_flag() {
        let cli = Cli::try_parse_from(["astu", "-o", "json", "lookup"]).expect("parse");
        let debug = format!("{:?}", cli);
        assert!(debug.contains("Json"), "{debug}");
    }

    #[test]
    fn parses_action_stdin_and_target_file_flags() {
        let cli = Cli::try_parse_from([
            "astu",
            "run",
            "--stdin",
            "target",
            "--target-file",
            "-",
            "--target-file",
            "/tmp/targets.txt",
            "hi",
        ])
        .expect("parse");
        let debug = format!("{:?}", cli);
        assert!(debug.contains("Target"), "{debug}");
        assert!(debug.contains("/tmp/targets.txt"), "{debug}");
    }
}
