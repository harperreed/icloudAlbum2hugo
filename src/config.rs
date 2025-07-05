//! Configuration management for icloud2hugo.
//!
//! This module handles loading and saving the application configuration.
//! It defines the `Config` struct that holds settings like album URLs and output directories,
//! and provides methods to read from and write to YAML files.
//!
//! The configuration supports multiple outputs, allowing photos to be synced from
//! different albums into different directories, either as photostreams or galleries.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// Define constants for default configuration values for clarity and reusability
const DEFAULT_ALBUM_URL: &str = "https://www.icloud.com/sharedalbum/ALBUM_TOKEN_GOES_HERE";
const DEFAULT_OUT_DIR: &str = "content/photostream";
const DEFAULT_DATA_FILE: &str = "data/photos/index.yaml";
const DEFAULT_FUZZ_METERS: f64 = 100.0;
const DEFAULT_CONFIG_FILE: &str = "icloudalbums.yaml";

// Define separate constants for test data - explicitly for testing only
#[cfg(test)]
pub(crate) mod test_constants {
    #[allow(dead_code)]
    pub const TEST_ALBUM_TOKEN: &str = "B0T3STt0k3n123456";
    #[allow(dead_code)]
    pub const TEST_ALBUM_URL: &str = "https://www.icloud.com/sharedalbum/#B0T3STt0k3n123456";
    #[allow(dead_code)]
    pub const TEST_MOCK_DIRECTORY: &str = "custom/tests";
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    Photostream,
    Gallery,
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Photostream
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PrivacyConfig {
    /// Whether to exclude from RSS feeds
    #[serde(default)]
    pub nofeed: bool,
    /// Whether to exclude from search engine indexing
    #[serde(default)]
    pub noindex: bool,
    /// Whether to use UUID-based slugs instead of human-readable ones
    #[serde(default)]
    pub uuid_slug: bool,
    /// Whether to mark as unlisted (not shown in listings)
    #[serde(default)]
    pub unlisted: bool,
    /// Whether to add robots meta tag with noindex, nofollow
    #[serde(default)]
    pub robots_noindex: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConfig {
    /// Type of output (photostream or gallery)
    #[serde(default)]
    pub output_type: OutputType,
    /// Album URL for this output
    pub album_url: String,
    /// Output directory for this output
    pub out_dir: String,
    /// Data file for this output
    pub data_file: String,
    /// Optional name for this output (if not provided, album name will be used)
    pub name: Option<String>,
    /// Optional description for this output
    pub description: Option<String>,
    /// Whether this output is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Privacy settings for Hugo frontmatter
    #[serde(default)]
    pub privacy: PrivacyConfig,
}

fn default_enabled() -> bool {
    true
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_type: OutputType::Photostream,
            album_url: DEFAULT_ALBUM_URL.to_string(),
            out_dir: DEFAULT_OUT_DIR.to_string(),
            data_file: DEFAULT_DATA_FILE.to_string(),
            name: None,
            description: None,
            enabled: true,
            privacy: PrivacyConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Default fuzz meters for location privacy
    pub fuzz_meters: Option<f64>,
    /// List of outputs to process
    #[serde(default)]
    pub outputs: Vec<OutputConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fuzz_meters: Some(DEFAULT_FUZZ_METERS),
            outputs: vec![OutputConfig::default()],
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

        let mut config: Config = serde_yaml::from_str(&yaml)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;

        // Handle legacy config format with single album_url/out_dir/data_file
        if config.outputs.is_empty() && yaml.contains("album_url") {
            // Parse as the old format first
            #[derive(Debug, Serialize, Deserialize)]
            struct LegacyConfig {
                pub album_url: String,
                pub out_dir: String,
                pub data_file: String,
                pub fuzz_meters: Option<f64>,
            }

            if let Ok(legacy_config) = serde_yaml::from_str::<LegacyConfig>(&yaml) {
                // Convert legacy format to new format
                config.fuzz_meters = legacy_config.fuzz_meters;
                config.outputs = vec![OutputConfig {
                    output_type: OutputType::Photostream,
                    album_url: legacy_config.album_url,
                    out_dir: legacy_config.out_dir,
                    data_file: legacy_config.data_file,
                    name: None,
                    description: None,
                    enabled: true,
                    privacy: PrivacyConfig::default(),
                }];
            }
        }

        // Ensure there's at least one output defined
        if config.outputs.is_empty() {
            config.outputs.push(OutputConfig::default());
        }

        Ok(config)
    }

    pub fn get_config_path(config_arg: &Option<PathBuf>) -> PathBuf {
        config_arg
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE))
    }

    /// Get only the enabled outputs
    pub fn enabled_outputs(&self) -> Vec<&OutputConfig> {
        self.outputs
            .iter()
            .filter(|output| output.enabled)
            .collect()
    }

    /// Get outputs by name
    pub fn get_outputs_by_name(&self, names: &[String]) -> Vec<&OutputConfig> {
        if names.is_empty() {
            return self.enabled_outputs();
        }

        self.outputs
            .iter()
            .filter(|output| {
                output.enabled
                    && output
                        .name
                        .as_ref()
                        .is_some_and(|name| names.contains(name))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.fuzz_meters, Some(DEFAULT_FUZZ_METERS));
        assert_eq!(config.outputs.len(), 1);

        let default_output = &config.outputs[0];
        assert!(matches!(
            default_output.output_type,
            OutputType::Photostream
        ));
        assert_eq!(default_output.album_url, DEFAULT_ALBUM_URL);
        assert_eq!(default_output.out_dir, DEFAULT_OUT_DIR);
        assert_eq!(default_output.data_file, DEFAULT_DATA_FILE);
        assert!(default_output.enabled);
    }

    #[test]
    fn test_save_and_load_config() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("icloudalbums.yaml");

        let config = Config::default();
        config.save_to_file(&config_path)?;

        let loaded_config = Config::load_from_file(&config_path)?;

        assert_eq!(config.fuzz_meters, loaded_config.fuzz_meters);
        assert_eq!(config.outputs.len(), loaded_config.outputs.len());

        let original_output = &config.outputs[0];
        let loaded_output = &loaded_config.outputs[0];

        assert!(matches!(loaded_output.output_type, OutputType::Photostream));
        assert_eq!(original_output.album_url, loaded_output.album_url);
        assert_eq!(original_output.out_dir, loaded_output.out_dir);
        assert_eq!(original_output.data_file, loaded_output.data_file);
        assert_eq!(original_output.enabled, loaded_output.enabled);

        Ok(())
    }

    #[test]
    fn test_load_legacy_config() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("legacy_config.yaml");

        // Create a legacy format config file
        let legacy_yaml = r#"
album_url: https://www.icloud.com/sharedalbum/legacy_token
out_dir: content/legacy
data_file: data/legacy.yaml
fuzz_meters: 50.0
"#;
        fs::write(&config_path, legacy_yaml)?;

        // Load it with the new code
        let config = Config::load_from_file(&config_path)?;

        // Verify it was converted correctly
        assert_eq!(config.fuzz_meters, Some(50.0));
        assert_eq!(config.outputs.len(), 1);

        let output = &config.outputs[0];
        assert!(matches!(output.output_type, OutputType::Photostream));
        assert_eq!(
            output.album_url,
            "https://www.icloud.com/sharedalbum/legacy_token"
        );
        assert_eq!(output.out_dir, "content/legacy");
        assert_eq!(output.data_file, "data/legacy.yaml");
        assert!(output.enabled);

        Ok(())
    }

    #[test]
    fn test_enabled_outputs() -> Result<()> {
        let mut config = Config::default();

        // Add a second disabled output
        let mut second_output = OutputConfig::default();
        second_output.album_url = "https://example.com/album2".to_string();
        second_output.enabled = false;
        config.outputs.push(second_output);

        // Add a third enabled output
        let mut third_output = OutputConfig::default();
        third_output.album_url = "https://example.com/album3".to_string();
        third_output.name = Some("Third Album".to_string());
        config.outputs.push(third_output);

        // Check that only enabled outputs are returned
        let enabled = config.enabled_outputs();
        assert_eq!(enabled.len(), 2);
        assert_eq!(enabled[0].album_url, DEFAULT_ALBUM_URL);
        assert_eq!(enabled[1].album_url, "https://example.com/album3");

        // Test filtering by name
        let named = config.get_outputs_by_name(&["Third Album".to_string()]);
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].album_url, "https://example.com/album3");

        Ok(())
    }

    #[test]
    fn test_privacy_config_default() {
        let privacy = PrivacyConfig::default();

        assert!(!privacy.nofeed);
        assert!(!privacy.noindex);
        assert!(!privacy.uuid_slug);
        assert!(!privacy.unlisted);
        assert!(!privacy.robots_noindex);
    }

    #[test]
    fn test_privacy_config_serialization() -> Result<()> {
        let mut privacy = PrivacyConfig::default();
        privacy.nofeed = true;
        privacy.uuid_slug = true;

        let yaml = serde_yaml::to_string(&privacy)?;
        let deserialized: PrivacyConfig = serde_yaml::from_str(&yaml)?;

        assert!(deserialized.nofeed);
        assert!(!deserialized.noindex);
        assert!(deserialized.uuid_slug);
        assert!(!deserialized.unlisted);
        assert!(!deserialized.robots_noindex);

        Ok(())
    }

    #[test]
    fn test_output_config_with_privacy() -> Result<()> {
        let mut config = OutputConfig::default();
        config.privacy.nofeed = true;
        config.privacy.robots_noindex = true;

        let yaml = serde_yaml::to_string(&config)?;
        let deserialized: OutputConfig = serde_yaml::from_str(&yaml)?;

        assert!(deserialized.privacy.nofeed);
        assert!(deserialized.privacy.robots_noindex);
        assert!(!deserialized.privacy.noindex);

        Ok(())
    }
}
