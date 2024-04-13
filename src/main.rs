#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// mod cli;
// mod config;
// mod selection;
pub mod resolver;
pub mod target;

use anyhow::Result;

fn main() -> Result<()> {
    // crate::cli::Cli::new().run()
    Ok(())
}
