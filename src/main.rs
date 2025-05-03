use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "A tool to sync photos from iCloud to Hugo")]
struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize with a default config file
    Init,
    
    /// Sync photos from iCloud to Hugo
    Sync,
    
    /// Show status of photos
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            println!("Initializing config...");
            // Will implement config generation later
            Ok(())
        }
        Commands::Sync => {
            println!("Syncing photos...");
            // Will implement sync logic later
            Ok(())
        }
        Commands::Status => {
            println!("Checking status...");
            // Will implement status reporting later
            Ok(())
        }
    }
}
