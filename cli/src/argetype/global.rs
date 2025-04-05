use anyhow::Context;
use anyhow::Result;
use astu::util::dirs;
use astu_db::DbImpl;
use camino::Utf8PathBuf;
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

    /// Data directory
    #[clap(long, default_value_t = data_dir(), help_heading = HEADING, global = true)]
    pub data_dir: Utf8PathBuf,
}

impl GlobalArgs {
    /// Initializes all [`tracing`] config.
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

    /// Gets a ready database connection.
    pub async fn get_db(&self) -> Result<DbImpl> {
        let db_file = self.data_dir.join("astu.db");
        DbImpl::try_new(db_file.as_str())
            .await
            .context("unable to connect to a db")
    }
}

fn data_dir() -> Utf8PathBuf {
    dirs::data_dir("astu")
        .try_into()
        .expect("unable to get data dir")
}
