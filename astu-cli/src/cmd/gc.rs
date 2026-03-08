use clap::Args;

#[derive(Debug, Args)]
pub struct Gc {
    #[arg(long, value_name = "DURATION")]
    pub before: Option<String>,
}
