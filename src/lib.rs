#![allow(non_snake_case)]
//! # icloud2hugo
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

// Export modules for integration testing
pub mod api_debug;
pub mod config;
pub mod exif;
pub mod gallery;
pub mod geocode;
pub mod icloud;
pub mod index;
pub mod mock;
pub mod sync;

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::error::Error;
    use std::fs;
    use std::process::Command as StdCommand;
    use tempfile::TempDir;

    fn cargo_bin() -> Command {
        let cargo = StdCommand::new(env!("CARGO"))
            .arg("build")
            .output()
            .expect("Failed to build binary");

        assert!(cargo.status.success(), "Failed to build icloudAlbum2hugo");

        Command::cargo_bin("icloudAlbum2hugo").expect("Failed to find icloudAlbum2hugo binary")
    }

    #[test]
    fn test_config_generation() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("icloudalbums.yaml");

        // Create a config file with init command
        let mut cmd = cargo_bin();
        cmd.arg("init")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Check if config file exists
        assert!(config_path.exists(), "Config file should be created");

        // Read the config file content
        let content = fs::read_to_string(&config_path)?;
        assert!(content.contains("outputs"), "Config should contain outputs");
        assert!(
            content.contains("album_url"),
            "Config should contain album_url"
        );
        assert!(content.contains("out_dir"), "Config should contain out_dir");
        assert!(
            content.contains("data_file"),
            "Config should contain data_file"
        );

        Ok(())
    }

    #[test]
    fn test_init_command_with_force() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("icloudalbums.yaml");

        // Create initial config
        let initial_content = "outputs: []";
        fs::write(&config_path, initial_content)?;

        // Run init command without force (should not overwrite)
        let mut cmd = cargo_bin();
        let output = cmd
            .arg("init")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Check stdout for "already exists" message
        let stdout = String::from_utf8(output.get_output().stdout.clone())?;
        assert!(
            stdout.contains("Config file already exists"),
            "Should detect existing config"
        );

        // Check content wasn't changed
        let content = fs::read_to_string(&config_path)?;
        assert_eq!(
            content, initial_content,
            "Content should not be changed without --force"
        );

        // Run init command with force (should overwrite)
        let mut cmd = cargo_bin();
        cmd.arg("init")
            .arg("--force")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Check content was changed
        let new_content = fs::read_to_string(&config_path)?;
        assert_ne!(
            new_content, initial_content,
            "Content should be changed with --force"
        );
        assert!(
            new_content.contains("outputs"),
            "New config should contain outputs"
        );
        assert!(
            new_content.contains("album_url"),
            "New config should contain album_url"
        );

        Ok(())
    }

    #[test]
    fn test_init_with_custom_config_path() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let custom_path = temp_dir.path().join("custom_config.yaml");

        // Run init with custom config path
        let mut cmd = cargo_bin();
        cmd.arg("init")
            .arg("--config")
            .arg(&custom_path)
            .assert()
            .success();

        // Check custom config was created
        assert!(custom_path.exists(), "Custom config file should be created");

        Ok(())
    }

    #[test]
    fn test_sync_command() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("icloudalbums.yaml");

        // Create config file
        let config_content = r#"
fuzz_meters: 100.0
outputs:
  - output_type: photostream
    album_url: "https://www.icloud.com/sharedalbum/#test123"
    out_dir: "content/photostream" 
    data_file: "data/photos/index.yaml"
    enabled: true
"#;
        fs::write(&config_path, config_content)?;

        // Run sync command
        let mut cmd = cargo_bin();
        let output = cmd
            .arg("sync")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let stdout = String::from_utf8(output.get_output().stdout.clone())?;
        assert!(stdout.contains("Syncing photos"), "Should mention syncing");
        assert!(stdout.contains("Album URL:"), "Should show album URL");

        Ok(())
    }

    #[test]
    fn test_status_command() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("icloudalbums.yaml");

        // Create config file
        let config_content = r#"
fuzz_meters: 100.0
outputs:
  - output_type: photostream
    album_url: "https://www.icloud.com/sharedalbum/#test123"
    out_dir: "content/photostream"
    data_file: "data/photos/index.yaml" 
    enabled: true
"#;
        fs::write(&config_path, config_content)?;

        // Run status command
        let mut cmd = cargo_bin();
        let output = cmd
            .arg("status")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let stdout = String::from_utf8(output.get_output().stdout.clone())?;
        assert!(
            stdout.contains("icloudAlbum2hugo Status"),
            "Should show status header"
        );
        assert!(
            stdout.contains("Configuration:"),
            "Should show configuration section"
        );
        assert!(stdout.contains("Album URL:"), "Should show album URL");
        assert!(
            stdout.contains("Output directory:"),
            "Should show output directory"
        );
        assert!(stdout.contains("Data file:"), "Should show data file path");

        Ok(())
    }

    #[test]
    fn test_status_command_with_data() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("icloudalbums.yaml");
        let data_dir = temp_dir.path().join("data").join("photos");
        let index_path = data_dir.join("index.yaml");

        // Create directories
        fs::create_dir_all(&data_dir)?;

        // Create config file
        let config_content = format!(
            r#"
fuzz_meters: 100.0
outputs:
  - output_type: photostream
    album_url: "https://www.icloud.com/sharedalbum/#test123"
    out_dir: "{}/content/photostream"
    data_file: "{}"
    enabled: true
"#,
            temp_dir.path().display(),
            index_path.display()
        );
        fs::write(&config_path, config_content)?;

        // Create a simple index.yaml file
        let index_content = r#"
last_updated: 2023-01-01T00:00:00Z
photos:
  test1:
    guid: "test1"
    filename: "test1.jpg"
    caption: "Test Photo 1"
    created_at: 2023-01-01T00:00:00Z
    checksum: "abc123"
    url: "https://example.com/test1.jpg"
    width: 1200
    height: 800
    last_sync: 2023-01-01T00:00:00Z
    local_path: "content/photostream/test1/original.jpg"
    camera_make: "Test Make"
    camera_model: "Test Model"
    exif_date_time: 2023-01-01T00:00:00Z
    latitude: 41.8781
    longitude: -87.6298
    fuzzed_latitude: 41.8782
    fuzzed_longitude: -87.6299
    iso: 100
    exposure_time: "1/100"
    f_number: 2.8
    focal_length: 28.0
    location: 
      formatted_address: "Chicago, IL, USA"
      city: "Chicago"
      state: "Illinois"
      country: "United States"
galleries: {}
"#;
        fs::write(&index_path, index_content)?;

        // Run status command with custom config
        let mut cmd = cargo_bin();
        let output = cmd
            .arg("status")
            .arg("--config")
            .arg(&config_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let stdout = String::from_utf8(output.get_output().stdout.clone())?;

        // Check detailed information is displayed
        assert!(
            stdout.contains("Photo index loaded with 1 photos"),
            "Should show correct photo count"
        );
        assert!(
            stdout.contains("Photos with EXIF data: 1/1"),
            "Should show EXIF stats"
        );
        assert!(
            stdout.contains("Photos with GPS coordinates: 1/1"),
            "Should show GPS stats"
        );

        Ok(())
    }

    #[test]
    fn test_sync_with_custom_config() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let custom_path = temp_dir.path().join("custom_config.yaml");

        // Create config at custom path
        let config_content = r#"
fuzz_meters: 50.0
outputs:
  - output_type: photostream
    album_url: "https://www.icloud.com/sharedalbum/#custom123"
    out_dir: "custom/path"
    data_file: "custom/data.yaml"
    enabled: true
"#;
        fs::write(&custom_path, config_content)?;

        // Run sync with custom config
        let mut cmd = cargo_bin();
        let output = cmd
            .arg("sync")
            .arg("--config")
            .arg(&custom_path)
            .assert()
            .success();

        let stdout = String::from_utf8(output.get_output().stdout.clone())?;
        assert!(stdout.contains("Syncing photos"), "Should mention syncing");
        assert!(
            stdout.contains("custom/path"),
            "Should show custom output path"
        );

        Ok(())
    }

    #[test]
    fn test_missing_config_error() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let nonexistent_path = temp_dir.path().join("does_not_exist.yaml");

        // Run sync with nonexistent config path
        let mut cmd = cargo_bin();
        cmd.arg("sync")
            .arg("--config")
            .arg(&nonexistent_path)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Config file not found"));

        Ok(())
    }
}
