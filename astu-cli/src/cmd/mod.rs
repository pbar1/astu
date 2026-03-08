mod freq;
mod gc;
mod jobs;
mod lookup;
mod output;
mod ping;
mod resume;
mod run;
mod tasks;
mod trace;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[command(visible_aliases = ["l", "resolve"])]
    Lookup(lookup::Lookup),

    #[command(visible_alias = "p")]
    Ping(ping::Ping),

    #[command(visible_aliases = ["r", "exec"])]
    Run(run::Run),

    Resume(resume::Resume),

    #[command(visible_aliases = ["o", "out"])]
    Output(output::Output),

    #[command(visible_alias = "f")]
    Freq(freq::Freq),

    Trace(trace::Trace),

    #[command(visible_aliases = ["j", "job"])]
    Jobs(jobs::Jobs),

    #[command(visible_aliases = ["t", "task"])]
    Tasks(tasks::Tasks),

    Gc(gc::Gc),
}
