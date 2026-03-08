use std::path::PathBuf;

use clap::Args;
use clap::ValueEnum;

#[derive(Debug, Clone, Default, Args)]
pub struct GlobalFlags {
    #[arg(global = true, long, env = "ASTU_DATA_DIR", value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    #[arg(global = true, long, env = "ASTU_LOG", default_value = "debug")]
    pub log_level: String,

    #[arg(global = true, short = 'o', long, default_value = "text")]
    pub output: OutputFormat,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}
