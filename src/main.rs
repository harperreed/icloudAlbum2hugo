mod config;
mod icloud;
mod api_debug;
mod index;

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
            
            // Load or create the photo index
            let data_file_path = PathBuf::from(&config_data.data_file);
            println!("Loading photo index from {}...", data_file_path.display());
            let photo_index = index::PhotoIndex::load(&data_file_path)
                .context("Failed to load photo index")?;
            
            println!("Photo index loaded with {} photos", photo_index.photo_count());
            
            // Debug the API to understand its structure
            println!("Debugging album API...");
            debug_album_api(&config_data.album_url).await
                .context("Failed to debug album API")?;
            
            // For now, we'll just save the index back (no changes yet)
            // In a real implementation, we would:
            // 1. Fetch the album data
            // 2. Compare with local index
            // 3. Download new/updated photos
            // 4. Remove deleted photos
            // 5. Update the index
            println!("Saving photo index to {}...", data_file_path.display());
            photo_index.save(&data_file_path)
                .context("Failed to save photo index")?;
            
            println!("Sync completed successfully");
            
            Ok(())
        }
        Commands::Status { config } => {
            let config_data = load_config(config)?;
            println!("Checking status...");
            println!("Album URL: {}", config_data.album_url);
            println!("Output directory: {}", config_data.out_dir);
            println!("Data file: {}", config_data.data_file);
            
            // Load or create the photo index
            let data_file_path = PathBuf::from(&config_data.data_file);
            println!("Loading photo index from {}...", data_file_path.display());
            let photo_index = match index::PhotoIndex::load(&data_file_path) {
                Ok(index) => index,
                Err(err) => {
                    println!("Warning: Could not load photo index: {}", err);
                    println!("Using empty index instead");
                    index::PhotoIndex::new()
                }
            };
            
            println!("Local photos in index: {}", photo_index.photo_count());
            
            // Use the debug function to get remote album data
            println!("Using debug function to get album data...");
            debug_album_api(&config_data.album_url).await
                .context("Failed to debug album API")?;
            
            println!("\nStatus summary:");
            println!("  Local photos in index: {}", photo_index.photo_count());
            println!("  Last updated: {}", photo_index.last_updated);
            println!("  Remote status: See album_data_debug.txt for details");
            
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
