#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod cli;
mod selection;
mod target_types;

use anyhow::Result;

fn main() -> Result<()> {
    crate::cli::Cli::new().run()
}
