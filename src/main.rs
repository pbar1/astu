#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod cli;
mod selection;
mod target_types;

use anyhow::Result;

use crate::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::new();

    cli.run()
}
