#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(async_fn_in_trait)]

mod action;
mod args;
mod cmd;
mod field;
mod report;
mod runtime;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cmd::run().await
}
