#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(async_fn_in_trait)]

mod arggroup;
mod cmd;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cmd::run().await
}
