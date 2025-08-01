use std::any::Any;
use std::fs::OpenOptions;
use std::io::IsTerminal;

use anyhow::Context;
use anyhow::Result;
use astu::db::DbImpl;
use astu::util::dirs;
use camino::Utf8PathBuf;
use clap::Args;
use tracing::debug;
use tracing_appender::non_blocking;
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
    /// Filter directive for stderr logs
    #[clap(long, env = "RUST_LOG", default_value = "error", help_heading = HEADING, global = true)]
    pub log_level: String,

    /// Filter directive for log file
    #[clap(long, default_value = "astu=debug,astu_cli=debug", help_heading = HEADING, global = true)]
    pub file_level: String,

    /// Data directory
    #[clap(long, default_value_t = data_dir(), help_heading = HEADING, global = true)]
    pub data_dir: Utf8PathBuf,
}

/// Guard holder for [`tracing`] things that need to live until the end of the
/// program.
#[derive(Debug, Default)]
pub struct TracingGuard {
    guards: Vec<Box<dyn Any>>,
}

impl GlobalArgs {
    /// Initializes all [`tracing`] config.
    pub fn init_tracing(&self) -> Result<TracingGuard> {
        let mut guard = TracingGuard::default();

        let indicatif_layer = IndicatifLayer::new();

        let stderr_filter = EnvFilter::builder().parse_lossy(&self.log_level);
        let stderr_writer = indicatif_layer.get_stderr_writer();
        let stderr_layer = tracing_subscriber::fmt::layer()
            .event_format(Glog::default().with_timer(LocalTime::default()))
            .fmt_fields(GlogFields::default())
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(stderr_writer)
            .with_filter(stderr_filter);

        let log_file_path = self.data_dir.join("last.log");
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)
            .context("unable to create log file")?;
        let (file_writer, file_writer_guard) = non_blocking(log_file);
        let file_filter = EnvFilter::builder().parse_lossy(&self.file_level);
        let file_layer = tracing_subscriber::fmt::layer()
            .event_format(Glog::default().with_timer(LocalTime::default()))
            .fmt_fields(GlogFields::default())
            .with_ansi(false)
            .with_writer(file_writer)
            .with_filter(file_filter);
        guard.guards.push(Box::new(file_writer_guard));

        let subscriber = Registry::default()
            .with(stderr_layer)
            .with(file_layer)
            .with(indicatif_layer);
        tracing::subscriber::set_global_default(subscriber)?;

        debug!("Initialized tracing");

        Ok(guard)
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
