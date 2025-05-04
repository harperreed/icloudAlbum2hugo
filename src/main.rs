mod config;
mod icloud;
mod api_debug;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use config::Config;
use api_debug::debug_album_api;

#[derive(Parser)]
#[command(author, version, about = "A tool to sync photos from iCloud to Hugo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize with a default config file
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
        
        /// Path to config file
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    
    /// Sync photos from iCloud to Hugo
    Sync {
        /// Path to config file
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    
    /// Show status of photos
    Status {
        /// Path to config file
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { force, config } => {
            init_config(config, *force)?;
            Ok(())
        }
        Commands::Sync { config } => {
            let config_data = load_config(config)?;
            println!("Syncing photos...");
            println!("Album URL: {}", config_data.album_url);
            println!("Output directory: {}", config_data.out_dir);
            println!("Data file: {}", config_data.data_file);
            
            // Debug the API to understand its structure
            println!("Debugging album API...");
            debug_album_api(&config_data.album_url).await
                .context("Failed to debug album API")?;
            
            // Fetch the album data (temporarily disabled until we fix the implementation)
            println!("Note: Full implementation temporarily disabled while we explore the API");
            
            Ok(())
        }
        Commands::Status { config } => {
            let config_data = load_config(config)?;
            println!("Checking status...");
            println!("Album URL: {}", config_data.album_url);
            println!("Output directory: {}", config_data.out_dir);
            println!("Data file: {}", config_data.data_file);
            
            // Use the debug function
            println!("Using debug function to get album data...");
            debug_album_api(&config_data.album_url).await
                .context("Failed to debug album API")?;
                
            println!("Remote status obtained");
            println!("Local index not implemented yet");
            
            Ok(())
        }
    }
}

fn init_config(config_path_opt: &Option<PathBuf>, force: bool) -> Result<()> {
    let config_path = Config::get_config_path(config_path_opt);
    
    if config_path.exists() && !force {
        println!("Config file already exists at {}", config_path.display());
        println!("Use --force to overwrite");
        return Ok(());
    }
    
    let config = Config::default();
    config.save_to_file(&config_path)
        .with_context(|| format!("Failed to write config to {}", config_path.display()))?;
    
    println!("Created config file at {}", config_path.display());
    Ok(())
}

fn load_config(config_path_opt: &Option<PathBuf>) -> Result<Config> {
    let config_path = Config::get_config_path(config_path_opt);
    
    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found at {}. Run 'icloud2hugo init' to create one.",
            config_path.display()
        );
    }
    
    Config::load_from_file(&config_path)
}
