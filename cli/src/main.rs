#![feature(async_closure)]

mod argetype;
mod cmd;
mod mapper;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    cmd::run().await
}

fn init_tracing() {
    use tracing_glog::Glog;
    use tracing_glog::GlogFields;
    use tracing_glog::LocalTime;

    tracing_subscriber::fmt()
        .event_format(Glog::default().with_timer(LocalTime::default()))
        .fmt_fields(GlogFields::default())
        .init();
}
