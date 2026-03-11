use clap::Args;
use clap::ValueEnum;

use crate::arg::ActionFlags;

/// Run a command on targets
///
/// Performs this sequence of actions on each target in the set:
/// - Connect
/// - Auth (if required) (until authenticated)
/// - Exec
///
/// Persists stdout/stderr/exitcode/error, as well as the timing of each phase.
#[derive(Debug, Args)]
pub struct Run {
    #[command(flatten)]
    pub action: ActionFlags,

    /// Command template.
    #[arg(value_name = "COMMAND")]
    pub command: String,

    /// Stream task stdout and stderr to the terminal.
    #[arg(long)]
    pub live: bool,

    /// Deduplicators for line normalization.
    ///
    /// These values will be substituted for their template tokens when seen.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "TEMPLATE",
        default_values = ["param", "host", "user", "ip"]
    )]
    pub dedupe: Vec<TemplateToken>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TemplateToken {
    Param,
    Host,
    User,
    Ip,
}

impl crate::Run for Run {
    async fn run(&self) -> eyre::Result<()> {
        eyre::bail!("unimplemented")
    }
}
