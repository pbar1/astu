#![warn(clippy::pedantic)]

mod cli;
mod selection;
mod target_types;

use anyhow::Result;

use crate::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::new();

    cli.run()
}
