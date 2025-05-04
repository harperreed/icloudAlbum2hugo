//! Configuration management for icloud2hugo.
//!
//! This module handles loading and saving the application configuration.
//! It defines the `Config` struct that holds settings like album URL and output directories,
//! and provides methods to read from and write to YAML files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// Define constants for default configuration values for clarity and reusability
const DEFAULT_ALBUM_URL: &str = "https://www.icloud.com/sharedalbum/ALBUM_TOKEN_GOES_HERE";
const DEFAULT_OUT_DIR: &str = "content/photostream";
const DEFAULT_DATA_FILE: &str = "data/photos/index.yaml";
const DEFAULT_FUZZ_METERS: f64 = 100.0;

// Define separate constants for test data - explicitly for testing only
#[cfg(test)]
pub(crate) mod test_constants {
    pub const TEST_ALBUM_TOKEN: &str = "B0T3STt0k3n123456";
    pub const TEST_ALBUM_URL: &str = "https://www.icloud.com/sharedalbum/#B0T3STt0k3n123456";
    pub const TEST_MOCK_DIRECTORY: &str = "custom/tests";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub album_url: String,
    pub out_dir: String,
    pub data_file: String,
    pub fuzz_meters: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            album_url: DEFAULT_ALBUM_URL.to_string(),
            out_dir: DEFAULT_OUT_DIR.to_string(),
            data_file: DEFAULT_DATA_FILE.to_string(),
            fuzz_meters: Some(DEFAULT_FUZZ_METERS),
        }
    }
}

impl Config {
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;

        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let yaml = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let config = serde_yaml::from_str(&yaml)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;

        Ok(config)
    }

    pub fn get_config_path(config_arg: &Option<PathBuf>) -> PathBuf {
        config_arg
            .clone()
            .unwrap_or_else(|| PathBuf::from("config.yaml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.album_url, DEFAULT_ALBUM_URL);
        assert_eq!(config.out_dir, DEFAULT_OUT_DIR);
        assert_eq!(config.data_file, DEFAULT_DATA_FILE);
        assert_eq!(config.fuzz_meters, Some(DEFAULT_FUZZ_METERS));
    }

    #[test]
    fn test_save_and_load_config() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.yaml");

        let config = Config::default();
        config.save_to_file(&config_path)?;

        let loaded_config = Config::load_from_file(&config_path)?;

        assert_eq!(config.album_url, loaded_config.album_url);
        assert_eq!(config.out_dir, loaded_config.out_dir);
        assert_eq!(config.data_file, loaded_config.data_file);
        assert_eq!(config.fuzz_meters, loaded_config.fuzz_meters);

        Ok(())
    }
}
