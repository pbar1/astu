use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a command in a target
    Exec { target: Option<String> },
    /// Copy files and directories to and from targets
    Cp { target: Option<String> },
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Exec { target } => {
            println!("target: {target:?}");
        }
        Commands::Cp { target } => {
            println!("target: {target:?}");
        }
    }
}
