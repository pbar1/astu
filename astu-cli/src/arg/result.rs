use clap::Args;

#[derive(Debug, Clone, Default, Args)]
#[command(next_help_heading = "Result Flags")]
pub struct ResultFlags {
    #[arg(short = 'j', long, value_name = "JOB_ID")]
    pub job: Option<String>,
}
