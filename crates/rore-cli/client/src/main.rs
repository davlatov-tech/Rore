use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rore")]
#[command(about = "Rore UI Build Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Rore project
    New { name: String },
    /// Run the project on a device
    Run { platform: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name } => {
            println!("Creating new Rore project: {}", name);
        }
        Commands::Run { platform } => {
            println!("Building for platform: {}", platform);
            rore_core::init();
        }
    }
}