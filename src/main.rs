//! # icloudAlbum2hugo
//!
//! A command-line tool that syncs photos from iCloud Shared Albums to a Hugo site.
//!
//! This tool fetches photos from a shared iCloud album, extracts EXIF data,
//! performs reverse geocoding (when location data is available), and organizes everything
//! into Hugo page bundles under `content/photostream/<photo_id>/`.
//!
//! ## Features
//!
//! - Downloads new/updated photos at full resolution
//! - Removes photos that no longer exist in the album
//! - Extracts EXIF metadata (camera info, date/time, location)
//! - Reverse geocoding and location fuzzing for privacy
//! - Creates Hugo page bundles with proper frontmatter
//! - Maintains a master YAML index file

mod api_debug;
mod config;
mod exif;
mod gallery;
mod geocode;
mod icloud;
mod index;
mod mock;
mod sync;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::{Config, OutputType};
use gallery::GallerySyncer;
use icloud::fetch_album;
use log::{debug, error, info, warn};
use std::fs;
use std::path::PathBuf;
use sync::Syncer;

// Helper function to both log a message and print it to the console for user feedback
fn console_log(message: &str, level: log::Level) {
    match level {
        log::Level::Error => {
            error!("{}", message);
            println!("❌ {}", message);
        }
        log::Level::Warn => {
            warn!("{}", message);
            println!("⚠️  {}", message);
        }
        log::Level::Info => {
            info!("{}", message);
            println!("{}", message);
        }
        log::Level::Debug => {
            debug!("{}", message);
            // No console output for debug messages
        }
        log::Level::Trace => {
            log::trace!("{}", message);
            // No console output for trace messages
        }
    }
}

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A tool to sync photos from iCloud Shared Albums to Hugo"
)]
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

        /// Only process outputs with these names
        #[arg(short, long)]
        output: Option<Vec<String>>,
    },

    /// Show status of photos
    Status {
        /// Path to config file
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Only show status for outputs with these names
        #[arg(short, long)]
        output: Option<Vec<String>>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Seconds))
        .init();

    info!("Starting icloudAlbum2hugo");
    debug!("Initialized logger");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { force, config } => {
            init_config(config, *force).context("Failed to initialize configuration")?;
            Ok(())
        }
        Commands::Sync { config, output } => {
            // ------- LOAD CONFIGURATION -------
            let config_data = load_config(config).context("Failed to load configuration")?;

            println!("┌─────────────────────────────────────────────┐");
            println!("│        icloudAlbum2hugo Photo Sync         │");
            println!("└─────────────────────────────────────────────┘");

            // Get the outputs to process
            let outputs_to_process = match &output {
                Some(names) => config_data.get_outputs_by_name(names),
                None => config_data.enabled_outputs(),
            };

            if outputs_to_process.is_empty() {
                println!("\n⚠️  No outputs found to process. Check your configuration.");
                return Ok(());
            }

            println!("\n📋 Configuration:");
            println!(
                "  • Found {} output(s) to process",
                outputs_to_process.len()
            );
            println!(
                "  • Location fuzz amount: {:?} meters",
                config_data.fuzz_meters
            );

            // Process each output
            for output_config in outputs_to_process {
                let output_name =
                    output_config
                        .name
                        .as_deref()
                        .unwrap_or(match output_config.output_type {
                            OutputType::Photostream => "Photostream",
                            OutputType::Gallery => "Gallery",
                        });

                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("🔄 Processing output: {}", output_name);
                println!("  • Type: {:?}", output_config.output_type);
                println!("  • Album URL: {}", output_config.album_url);
                println!("  • Output directory: {}", output_config.out_dir);
                println!("  • Data file: {}", output_config.data_file);

                // ------- LOAD PHOTO INDEX -------
                let data_file_path = PathBuf::from(&output_config.data_file);
                println!(
                    "\n📂 Loading photo index from {}...",
                    data_file_path.display()
                );

                let mut photo_index = match index::PhotoIndex::load(&data_file_path) {
                    Ok(index) => {
                        println!("  • Photo index loaded with {} photos", index.photo_count());
                        if index.gallery_count() > 0 {
                            println!("  • Index contains {} galleries", index.gallery_count());
                        }
                        index
                    }
                    Err(err) => {
                        eprintln!("  ⚠️  Warning: Could not load photo index: {}", err);
                        println!("  ℹ️  Creating new empty index");
                        index::PhotoIndex::new()
                    }
                };

                // ------- FETCH ALBUM DATA -------
                println!("\n🔄 Fetching album data from iCloud...");
                let album = match fetch_album(&output_config.album_url).await {
                    Ok(album) => {
                        println!(
                            "  • Album '{}' fetched with {} photos",
                            album.name,
                            album.photos.len()
                        );
                        album
                    }
                    Err(err) => {
                        eprintln!("  ⚠️  Error: Failed to fetch album data: {}", err);
                        println!("  ℹ️  Skipping this output and continuing with others");
                        continue;
                    }
                };

                // ------- PREPARE FOR SYNC -------
                let content_dir = PathBuf::from(&output_config.out_dir);

                // Process according to output type
                let results = match output_config.output_type {
                    OutputType::Photostream => {
                        // Use the existing Syncer for photostream
                        println!("\n📷 Syncing photos to photostream...");
                        let syncer = Syncer::new(content_dir, data_file_path.clone());
                        syncer
                            .sync_photos(&album, &mut photo_index)
                            .await
                            .context("Failed to sync photostream")?
                    }
                    OutputType::Gallery => {
                        // Use the new GallerySyncer for gallery
                        println!("\n🖼️  Creating gallery page bundle...");
                        let gallery_syncer = GallerySyncer::new(
                            content_dir,
                            output_config.name.clone(),
                            output_config.description.clone(), 
                            data_file_path.clone(),
                        );
                        gallery_syncer
                            .sync_gallery(&album, &mut photo_index)
                            .await
                            .context("Failed to sync gallery")?
                    }
                };

                // ------- COUNT RESULTS -------
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
                        sync::SyncResult::Failed(guid, error) => {
                            eprintln!("  ⚠️  Failed to sync photo {}: {}", guid, error);
                            failed += 1;
                        }
                    }
                }

                // ------- SAVE UPDATED INDEX -------
                println!("\n💾 Saving photo index to {}...", data_file_path.display());
                match photo_index.save(&data_file_path) {
                    Ok(_) => println!("  • Photo index saved successfully"),
                    Err(err) => {
                        eprintln!("  ⚠️  Warning: Failed to save photo index: {}", err);
                        eprintln!(
                            "  ℹ️  Your changes have been applied but not saved to the index file"
                        );
                    }
                }

                // ------- PRINT SUMMARY -------
                println!("\n✅ Sync completed for {}", output_name);
                println!("  • Added: {}", added);
                println!("  • Updated: {}", updated);
                println!("  • Unchanged: {}", unchanged);
                println!("  • Deleted: {}", deleted);
                if failed > 0 {
                    println!("  • Failed: {} (see warnings above)", failed);
                }
                println!("  • Total photos in index: {}", photo_index.photo_count());
            }

            println!("\n🎉 All outputs processed successfully!");
            Ok(())
        }
        Commands::Status { config, output } => {
            // ------- LOAD CONFIGURATION -------
            let config_data = load_config(config).context("Failed to load configuration")?;

            println!("┌─────────────────────────────────────────────┐");
            println!("│            icloudAlbum2hugo Status          │");
            println!("└─────────────────────────────────────────────┘");

            // Get the outputs to check
            let outputs_to_check = match &output {
                Some(names) => config_data.get_outputs_by_name(names),
                None => config_data.enabled_outputs(),
            };

            if outputs_to_check.is_empty() {
                println!("\n⚠️  No outputs found to check. Check your configuration.");
                return Ok(());
            }

            println!("\n📋 Configuration:");
            println!("  • Found {} output(s) to check", outputs_to_check.len());
            println!(
                "  • Location fuzz amount: {:?} meters",
                config_data.fuzz_meters
            );

            // Process each output
            for output_config in outputs_to_check {
                let output_name =
                    output_config
                        .name
                        .as_deref()
                        .unwrap_or(match output_config.output_type {
                            OutputType::Photostream => "Photostream",
                            OutputType::Gallery => "Gallery",
                        });

                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("🔍 Checking output: {}", output_name);
                println!("  • Type: {:?}", output_config.output_type);
                println!("  • Album URL: {}", output_config.album_url);
                println!("  • Output directory: {}", output_config.out_dir);
                println!("  • Data file: {}", output_config.data_file);

                // ------- LOAD PHOTO INDEX -------
                let data_file_path = PathBuf::from(&output_config.data_file);
                println!(
                    "\n📂 Loading photo index from {}...",
                    data_file_path.display()
                );
                let photo_index = match index::PhotoIndex::load(&data_file_path) {
                    Ok(index) => {
                        println!("  • Photo index loaded with {} photos", index.photo_count());
                        if index.gallery_count() > 0 {
                            println!("  • Index contains {} galleries", index.gallery_count());
                        }
                        index
                    }
                    Err(err) => {
                        eprintln!("  ⚠️  Warning: Could not load photo index: {}", err);
                        println!("  ℹ️  Using empty index instead");
                        index::PhotoIndex::new()
                    }
                };

                // ------- DISPLAY LOCAL INDEX STATS -------
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

                    println!(
                        "  • Photos with EXIF data: {}/{}",
                        exif_count,
                        photo_index.photo_count()
                    );
                    println!(
                        "  • Photos with GPS coordinates: {}/{}",
                        gps_count,
                        photo_index.photo_count()
                    );
                    println!(
                        "  • Photos with location info: {}/{}",
                        geocoded_count,
                        photo_index.photo_count()
                    );

                    // Show gallery info if this is a gallery output
                    if let OutputType::Gallery = output_config.output_type {
                        if photo_index.gallery_count() > 0 {
                            println!("\n🖼️  Gallery Information:");
                            for gallery in photo_index.galleries.values() {
                                println!(
                                    "  • Gallery '{}' contains {} photos",
                                    gallery.name,
                                    gallery.photos.len()
                                );
                            }
                        }
                    }
                }

                // ------- FETCH REMOTE ALBUM DATA -------
                println!("\n🔄 Fetching album data from iCloud...");
                let album = match fetch_album(&output_config.album_url).await {
                    Ok(album) => {
                        println!(
                            "  • Album '{}' fetched with {} photos",
                            album.name,
                            album.photos.len()
                        );
                        Some(album)
                    }
                    Err(err) => {
                        eprintln!("  ⚠️  Warning: Could not fetch album: {}", err);
                        eprintln!("    Error details: {}", err);
                        println!("  ℹ️  Status will only show local information");
                        None
                    }
                };

                // ------- COMPARE LOCAL AND REMOTE DATA -------
                if let Some(album) = album {
                    // Get the set of photo IDs from both sources
                    let remote_ids: std::collections::HashSet<&String> =
                        album.photos.keys().collect();
                    let local_ids: std::collections::HashSet<&String> =
                        photo_index.photos.keys().collect();

                    // Calculate the sets of new, common, and removed photos
                    let new_ids: Vec<&&String> = remote_ids.difference(&local_ids).collect();
                    let common_ids: Vec<&&String> = remote_ids.intersection(&local_ids).collect();
                    let removed_ids: Vec<&&String> = local_ids.difference(&remote_ids).collect();

                    // Count potential updates by comparing checksums
                    let mut update_count = 0;
                    let mut updated_ids = Vec::new();
                    for &&id in &common_ids {
                        let remote_photo = album
                            .photos
                            .get(id)
                            .expect("Photo should exist in remote album");
                        let local_photo = photo_index
                            .photos
                            .get(id)
                            .expect("Photo should exist in local index");

                        if remote_photo.checksum != local_photo.checksum {
                            update_count += 1;
                            updated_ids.push(id);
                        }
                    }

                    // ------- DISPLAY STATUS SUMMARY -------
                    println!("\n📊 Status Summary for {}:", output_name);
                    println!("  • Local photos: {}", photo_index.photos.len());
                    println!("  • Remote photos: {}", album.photos.len());
                    println!("  • Photos in sync: {}", common_ids.len() - update_count);
                    println!("  • New photos to download: {}", new_ids.len());
                    println!("  • Photos to update: {}", update_count);
                    println!("  • Photos to remove: {}", removed_ids.len());

                    // Show detailed information if requested
                    let show_detail = true; // Could be a command-line flag in the future

                    // ------- DISPLAY DETAILED PHOTO LISTS -------
                    if show_detail {
                        if !new_ids.is_empty() {
                            println!("\n🆕 New photos to download:");
                            for (i, &&id) in new_ids.iter().enumerate().take(5) {
                                let photo = album
                                    .photos
                                    .get(id)
                                    .expect("Photo should exist in remote album");
                                let caption = photo
                                    .caption
                                    .clone()
                                    .unwrap_or_else(|| "No caption".to_string());
                                println!("  {}. {} - {}", i + 1, id, caption);
                            }
                            if new_ids.len() > 5 {
                                println!("  ... and {} more", new_ids.len() - 5);
                            }
                        }

                        if !updated_ids.is_empty() {
                            println!("\n🔄 Photos to update:");
                            for (i, &id) in updated_ids.iter().enumerate().take(5) {
                                let photo = album
                                    .photos
                                    .get(id)
                                    .expect("Photo should exist in remote album");
                                let caption = photo
                                    .caption
                                    .clone()
                                    .unwrap_or_else(|| "No caption".to_string());
                                println!("  {}. {} - {}", i + 1, id, caption);
                            }
                            if updated_ids.len() > 5 {
                                println!("  ... and {} more", updated_ids.len() - 5);
                            }
                        }

                        if !removed_ids.is_empty() {
                            println!("\n🗑️  Photos to remove:");
                            for (i, &&id) in removed_ids.iter().enumerate().take(5) {
                                if let Some(photo) = photo_index.photos.get(id) {
                                    let caption = photo
                                        .caption
                                        .clone()
                                        .unwrap_or_else(|| "No caption".to_string());
                                    println!("  {}. {} - {}", i + 1, id, caption);
                                } else {
                                    println!("  {}. {}", i + 1, id);
                                }
                            }
                            if removed_ids.len() > 5 {
                                println!("  ... and {} more", removed_ids.len() - 5);
                            }
                        }
                    }

                    // ------- PROVIDE RECOMMENDATIONS -------
                    println!("\n📋 Suggested Actions:");
                    if new_ids.is_empty() && update_count == 0 && removed_ids.is_empty() {
                        println!("  ✅ Everything is up to date! No action needed.");
                    } else {
                        println!("  • Run 'icloudAlbum2hugo sync' to update your local files");
                        if output_config.name.is_some() {
                            println!(
                                "  • To update only this output: icloudAlbum2hugo sync -o \"{}\"",
                                output_config.name.as_ref().unwrap()
                            );
                        }
                    }
                } else {
                    // ------- LOCAL-ONLY SUMMARY -------
                    println!("\n📊 Status Summary for {} (local only):", output_name);
                    println!("  • Local photos: {}", photo_index.photos.len());
                    if photo_index.photo_count() > 0 {
                        println!("  • Last updated: {}", photo_index.last_updated);
                    }
                    println!("\n⚠️  Unable to compare with remote album data");
                    println!("  • Please check your internet connection and album URL");
                    println!("  • Verify that the album URL in your config is correct");
                }
            }

            println!("\n🎉 Status check completed for all outputs!");
            Ok(())
        }
    }
}

/// Initialize the configuration file
fn init_config(config_path_opt: &Option<PathBuf>, force: bool) -> Result<()> {
    let config_path = Config::get_config_path(config_path_opt);

    if config_path.exists() && !force {
        console_log(
            &format!("📋 Config file already exists at {}", config_path.display()),
            log::Level::Info,
        );
        console_log("   Use --force to overwrite", log::Level::Info);
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            debug!("Creating parent directory: {}", parent.display());
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
    }

    // Create a default config with example outputs
    let mut config = Config::default();

    // Add a gallery example (disabled by default)
    let gallery_example = config::OutputConfig {
        output_type: config::OutputType::Gallery,
        album_url: "https://www.icloud.com/sharedalbum/GALLERY_TOKEN_GOES_HERE".to_string(),
        out_dir: "content/galleries/my_gallery".to_string(),
        data_file: "data/photos/gallery.yaml".to_string(),
        name: Some("My Gallery".to_string()),
        description: Some("A collection of photos from my album".to_string()),
        enabled: false, // Disabled by default
    };

    config.outputs.push(gallery_example);

    debug!("Saving default configuration to {}", config_path.display());
    config
        .save_to_file(&config_path)
        .with_context(|| format!("Failed to write config to {}", config_path.display()))?;

    console_log(
        &format!("✅ Created config file at {}", config_path.display()),
        log::Level::Info,
    );
    console_log(
        "   Please edit this file to set your iCloud shared album URLs",
        log::Level::Info,
    );
    console_log(
        "   The config includes examples for both photostream and gallery outputs",
        log::Level::Info,
    );
    Ok(())
}

/// Load configuration from file
fn load_config(config_path_opt: &Option<PathBuf>) -> Result<Config> {
    let config_path = Config::get_config_path(config_path_opt);

    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found at {}.\nRun 'icloudAlbum2hugo init' to create one.",
            config_path.display()
        );
    }

    Config::load_from_file(&config_path).with_context(|| {
        format!(
            "Failed to load configuration from {}",
            config_path.display()
        )
    })
}
