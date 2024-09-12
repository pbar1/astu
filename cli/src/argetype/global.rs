use anyhow::Result;
use clap::Args;
use tracing_glog::Glog;
use tracing_glog::GlogFields;
use tracing_glog::LocalTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

const HEADING: Option<&str> = Some("Global Options");

/// Global arguments that apply to every subcommand.
#[derive(Debug, Args, Clone)]
pub struct GlobalArgs {
    /// Log level
    #[arg(long, env = "RUST_LOG", default_value = "error", help_heading = HEADING, global = true)]
    pub log_level: String,
}

impl GlobalArgs {
    pub fn init_tracing(&self) -> Result<()> {
        let glog_filter = EnvFilter::builder().parse_lossy(&self.log_level);
        let glog_layer = tracing_subscriber::fmt::layer()
            .event_format(Glog::default().with_timer(LocalTime::default()))
            .fmt_fields(GlogFields::default())
            .with_filter(glog_filter);

        let subscriber = Registry::default().with(glog_layer);
        tracing::subscriber::set_global_default(subscriber)?;

        Ok(())
    }
}
