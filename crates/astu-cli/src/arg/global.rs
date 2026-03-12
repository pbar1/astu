use std::path::PathBuf;

use clap::Args;
use clap::ValueEnum;

#[derive(Debug, Clone, Default, Args)]
pub struct GlobalFlags {
    /// Astu database path.
    #[arg(
        global = true,
        help_heading = "Global Flags",
        long,
        env = "ASTU_DATA_DIR",
        value_name = "DIR"
    )]
    pub data_dir: Option<PathBuf>,

    /// Filter directive for log file. Follows the `RUST_LOG` format.
    #[arg(
        global = true,
        help_heading = "Global Flags",
        long,
        env = "ASTU_LOG",
        default_value = "debug"
    )]
    pub log_level: String,

    /// Output format.
    #[arg(
        global = true,
        help_heading = "Global Flags",
        short = 'o',
        long,
        default_value = "text",
        value_name = "FORMAT"
    )]
    pub output: OutputFormat,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}
