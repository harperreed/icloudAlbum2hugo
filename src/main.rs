mod config;
mod icloud;
mod api_debug;
mod index;
mod sync;
mod mock;
mod exif;
mod geocode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use config::Config;
use sync::Syncer;
use mock::mock_fetch_album;

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
            let mut photo_index = index::PhotoIndex::load(&data_file_path)
                .context("Failed to load photo index")?;
            
            println!("Photo index loaded with {} photos", photo_index.photo_count());
            
            // Fetch the album data
            println!("Fetching album data from iCloud...");
            let album = mock_fetch_album(&config_data.album_url).await
                .context("Failed to fetch album data")?;
            
            println!("Album '{}' fetched with {} photos", album.name, album.photos.len());
            
            // Create a syncer
            let content_dir = PathBuf::from(&config_data.out_dir);
            let syncer = Syncer::new(content_dir, data_file_path.clone());
            
            // Sync photos
            println!("Syncing photos to local filesystem...");
            let results = syncer.sync_photos(&album, &mut photo_index).await
                .context("Failed to sync photos")?;
            
            // Count results by type
            let mut added = 0;
            let mut updated = 0;
            let mut unchanged = 0;
            let mut deleted = 0;
            let mut failed = 0;
            
            for result in &results {
                match result {
                    sync::SyncResult::Added(_) => added += 1,
                    sync::SyncResult::Updated(_) => updated += 1,
                    sync::SyncResult::Unchanged(_) => unchanged += 1,
                    sync::SyncResult::Deleted(_) => deleted += 1,
                    sync::SyncResult::Failed(_, _) => failed += 1,
                }
            }
            
            // Save the updated index
            println!("Saving photo index to {}...", data_file_path.display());
            syncer.save_index(&photo_index)
                .context("Failed to save photo index")?;
            
            println!("Sync completed successfully:");
            println!("  - Added: {}", added);
            println!("  - Updated: {}", updated);
            println!("  - Unchanged: {}", unchanged);
            println!("  - Deleted: {}", deleted);
            println!("  - Failed: {}", failed);
            println!("  - Total photos in index: {}", photo_index.photo_count());
            
            Ok(())
        }
        Commands::Status { config } => {
            let config_data = load_config(config)?;
            println!("┌─────────────────────────────────────────────┐");
            println!("│               icloud2hugo Status             │");
            println!("└─────────────────────────────────────────────┘");
            
            // Display configuration summary
            println!("\n📋 Configuration:");
            println!("  • Album URL: {}", config_data.album_url);
            println!("  • Output directory: {}", config_data.out_dir);
            println!("  • Data file: {}", config_data.data_file);
            
            // Load or create the photo index
            let data_file_path = PathBuf::from(&config_data.data_file);
            println!("\n📂 Loading photo index from {}...", data_file_path.display());
            let photo_index = match index::PhotoIndex::load(&data_file_path) {
                Ok(index) => index,
                Err(err) => {
                    println!("  ⚠️  Warning: Could not load photo index: {}", err);
                    println!("  ℹ️  Using empty index instead");
                    index::PhotoIndex::new()
                }
            };
            
            println!("  • Local photos in index: {}", photo_index.photo_count());
            
            if photo_index.photo_count() > 0 {
                println!("  • Last updated: {}", photo_index.last_updated);
                
                // Show metadata stats
                let mut exif_count = 0;
                let mut gps_count = 0;
                let mut geocoded_count = 0;
                
                for photo in photo_index.photos.values() {
                    if photo.camera_make.is_some() || photo.camera_model.is_some() {
                        exif_count += 1;
                    }
                    if photo.latitude.is_some() && photo.longitude.is_some() {
                        gps_count += 1;
                    }
                    if photo.location.is_some() {
                        geocoded_count += 1;
                    }
                }
                
                println!("  • Photos with EXIF data: {}/{}", exif_count, photo_index.photo_count());
                println!("  • Photos with GPS coordinates: {}/{}", gps_count, photo_index.photo_count());
                println!("  • Photos with location info: {}/{}", geocoded_count, photo_index.photo_count());
            }
            
            // Fetch the remote album
            println!("\n🔄 Fetching album data from iCloud...");
            let album = match mock_fetch_album(&config_data.album_url).await {
                Ok(album) => {
                    println!("  • Album '{}' fetched with {} photos", album.name, album.photos.len());
                    Some(album)
                },
                Err(err) => {
                    println!("  ⚠️  Warning: Could not fetch album: {}", err);
                    println!("  ℹ️  Status will only show local information");
                    None
                }
            };
            
            // If we have both local index and remote album, compare them
            if let Some(album) = album {
                // Get the set of photo IDs from both sources
                let remote_ids: std::collections::HashSet<&String> = album.photos.keys().collect();
                let local_ids: std::collections::HashSet<&String> = photo_index.photos.keys().collect();
                
                // Calculate the sets of new, common, and removed photos
                let new_ids: Vec<&&String> = remote_ids.difference(&local_ids).collect();
                let common_ids: Vec<&&String> = remote_ids.intersection(&local_ids).collect();
                let removed_ids: Vec<&&String> = local_ids.difference(&remote_ids).collect();
                
                // Count potential updates by comparing checksums
                let mut update_count = 0;
                let mut updated_ids = Vec::new();
                for &&id in &common_ids {
                    let remote_photo = album.photos.get(id).unwrap();
                    let local_photo = photo_index.photos.get(id).unwrap();
                    
                    if remote_photo.checksum != local_photo.checksum {
                        update_count += 1;
                        updated_ids.push(id);
                    }
                }
                
                println!("\n📊 Status Summary:");
                println!("  • Local photos: {}", photo_index.photos.len());
                println!("  • Remote photos: {}", album.photos.len());
                println!("  • Photos in sync: {}", common_ids.len() - update_count);
                println!("  • New photos to download: {}", new_ids.len());
                println!("  • Photos to update: {}", update_count);
                println!("  • Photos to remove: {}", removed_ids.len());
                
                // Show detailed information if requested
                let show_detail = true; // Could be a command-line flag in the future
                
                if show_detail {
                    if !new_ids.is_empty() {
                        println!("\n🆕 New photos to download:");
                        for (i, &&id) in new_ids.iter().enumerate().take(5) {
                            let photo = album.photos.get(id).unwrap();
                            let caption = photo.caption.clone().unwrap_or_else(|| "No caption".to_string());
                            println!("  {}. {} - {}", i+1, id, caption);
                        }
                        if new_ids.len() > 5 {
                            println!("  ... and {} more", new_ids.len() - 5);
                        }
                    }
                    
                    if !updated_ids.is_empty() {
                        println!("\n🔄 Photos to update:");
                        for (i, &id) in updated_ids.iter().enumerate().take(5) {
                            let photo = album.photos.get(id).unwrap();
                            let caption = photo.caption.clone().unwrap_or_else(|| "No caption".to_string());
                            println!("  {}. {} - {}", i+1, id, caption);
                        }
                        if updated_ids.len() > 5 {
                            println!("  ... and {} more", updated_ids.len() - 5);
                        }
                    }
                    
                    if !removed_ids.is_empty() {
                        println!("\n🗑️  Photos to remove:");
                        for (i, &&id) in removed_ids.iter().enumerate().take(5) {
                            if let Some(photo) = photo_index.photos.get(id) {
                                let caption = photo.caption.clone().unwrap_or_else(|| "No caption".to_string());
                                println!("  {}. {} - {}", i+1, id, caption);
                            } else {
                                println!("  {}. {}", i+1, id);
                            }
                        }
                        if removed_ids.len() > 5 {
                            println!("  ... and {} more", removed_ids.len() - 5);
                        }
                    }
                }
                
                // Suggested next steps
                println!("\n📋 Suggested Actions:");
                if new_ids.is_empty() && update_count == 0 && removed_ids.is_empty() {
                    println!("  ✅ Everything is up to date! No action needed.");
                } else {
                    println!("  • Run 'icloud2hugo sync' to update your local files");
                }
            } else {
                println!("\n📊 Status Summary (local only):");
                println!("  • Local photos: {}", photo_index.photos.len());
                if photo_index.photo_count() > 0 {
                    println!("  • Last updated: {}", photo_index.last_updated);
                }
                println!("\n⚠️  Unable to compare with remote album data");
                println!("  • Please check your internet connection and album URL");
            }
            
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
