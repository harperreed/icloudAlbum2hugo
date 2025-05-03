use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
            album_url: "https://www.icloud.com/sharedalbum/#...".to_string(),
            out_dir: "content/photostream".to_string(),
            data_file: "data/photos/index.yaml".to_string(),
            fuzz_meters: Some(100.0),
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
        
        assert_eq!(config.album_url, "https://www.icloud.com/sharedalbum/#...");
        assert_eq!(config.out_dir, "content/photostream");
        assert_eq!(config.data_file, "data/photos/index.yaml");
        assert_eq!(config.fuzz_meters, Some(100.0));
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