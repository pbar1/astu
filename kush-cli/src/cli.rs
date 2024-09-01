use clap::Parser;

/// Remote execution swiss army knife
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    Ok(())
}
