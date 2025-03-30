use anyhow::Result;
use clap::Args;
use tracing_glog::Glog;
use tracing_glog::GlogFields;
use tracing_glog::LocalTime;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

const HEADING: Option<&str> = Some("Global Options");

/// Global arguments that apply to every subcommand.
#[derive(Debug, Args, Clone)]
pub struct GlobalArgs {
    /// Log level
    #[clap(long, env = "RUST_LOG", default_value = "error", help_heading = HEADING, global = true)]
    pub log_level: String,
}

impl GlobalArgs {
    pub fn init_tracing(&self) -> Result<()> {
        let indicatif_layer = IndicatifLayer::new();

        let stderr_filter = EnvFilter::builder().parse_lossy(&self.log_level);
        let stderr_writer = indicatif_layer.get_stderr_writer();
        let stderr_layer = tracing_subscriber::fmt::layer()
            .event_format(Glog::default().with_timer(LocalTime::default()))
            .fmt_fields(GlogFields::default())
            .with_writer(stderr_writer)
            .with_filter(stderr_filter);

        let subscriber = Registry::default().with(stderr_layer).with(indicatif_layer);
        tracing::subscriber::set_global_default(subscriber)?;

        Ok(())
    }
}
