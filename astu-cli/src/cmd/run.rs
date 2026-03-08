use clap::{Args, ValueEnum};

use crate::arg::ActionFlags;

#[derive(Debug, Args)]
pub struct Run {
    #[command(flatten)]
    pub action: ActionFlags,

    #[arg(value_name = "COMMAND")]
    pub command: String,

    #[arg(long)]
    pub live: bool,

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
