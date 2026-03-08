use clap::Args;
use clap::ValueEnum;

#[derive(Debug, Clone, Default, Args)]
#[command(next_help_heading = "Action Flags")]
pub struct ActionFlags {
    #[arg(short = 'T', long, value_name = "TARGET")]
    pub target: Vec<String>,

    #[arg(short = 'f', long, value_name = "PATH")]
    pub target_file: Vec<String>,

    #[arg(long, default_value = "auto")]
    pub stdin: StdinMode,

    #[arg(long, default_value = "30s", value_name = "DURATION")]
    pub timeout: String,

    #[arg(long, value_name = "TARGETS")]
    pub confirm: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum StdinMode {
    #[default]
    Auto,
    Param,
    Target,
    Pipe,
}
