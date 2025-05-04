use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a stored photo's metadata in our local index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedPhoto {
    /// Unique identifier from iCloud
    pub guid: String,
    /// Original filename
    pub filename: String,
    /// Photo caption/description (if any)
    pub caption: Option<String>,
    /// When the photo was created
    pub created_at: DateTime<Utc>,
    /// Checksum of photo content (for detecting changes)
    pub checksum: String,
    /// URL to download the photo (may change over time)
    pub url: String,
    /// Width of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
    /// When this photo was last synchronized
    pub last_sync: DateTime<Utc>,
    /// Local path to the photo
    pub local_path: PathBuf,
}

/// Represents our local database of photos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoIndex {
    /// When the index was last updated
    pub last_updated: DateTime<Utc>,
    /// Map of photo GUIDs to indexed photos
    pub photos: HashMap<String, IndexedPhoto>,
}

impl PhotoIndex {
    /// Creates a new empty photo index
    pub fn new() -> Self {
        Self {
            last_updated: Utc::now(),
            photos: HashMap::new(),
        }
    }
    
    /// Load the photo index from the specified path
    pub fn load(path: &Path) -> Result<Self> {
        // If the file doesn't exist, create a new empty index
        if !path.exists() {
            return Ok(Self::new());
        }
        
        // Read and parse the YAML file
        let yaml = fs::read_to_string(path)
            .with_context(|| format!("Failed to read index file from {}", path.display()))?;
        
        let index: PhotoIndex = serde_yaml::from_str(&yaml)
            .with_context(|| format!("Failed to parse index file from {}", path.display()))?;
        
        Ok(index)
    }
    
    /// Save the photo index to the specified path
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory for {}", path.display()))?;
            }
        }
        
        // Serialize and write to file
        let yaml = serde_yaml::to_string(self)
            .with_context(|| "Failed to serialize index to YAML")?;
        
        fs::write(path, yaml)
            .with_context(|| format!("Failed to write index file to {}", path.display()))?;
        
        Ok(())
    }
    
    /// Add or update a photo in the index
    pub fn add_or_update_photo(&mut self, photo: IndexedPhoto) {
        self.photos.insert(photo.guid.clone(), photo);
        self.last_updated = Utc::now();
    }
    
    /// Remove a photo from the index
    pub fn remove_photo(&mut self, guid: &str) -> Option<IndexedPhoto> {
        let result = self.photos.remove(guid);
        if result.is_some() {
            self.last_updated = Utc::now();
        }
        result
    }
    
    /// Get a photo from the index by GUID
    pub fn get_photo(&self, guid: &str) -> Option<&IndexedPhoto> {
        self.photos.get(guid)
    }
    
    /// Number of photos in the index
    pub fn photo_count(&self) -> usize {
        self.photos.len()
    }
}

impl Default for PhotoIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts an iCloud photo to our indexed photo format
pub fn convert_to_indexed_photo(
    icloud_photo: &crate::icloud::Photo,
    content_dir: &Path,
    photo_id: &str,
) -> IndexedPhoto {
    IndexedPhoto {
        guid: icloud_photo.guid.clone(),
        filename: icloud_photo.filename.clone(),
        caption: icloud_photo.caption.clone(),
        created_at: icloud_photo.created_at,
        checksum: icloud_photo.checksum.clone(),
        url: icloud_photo.url.clone(),
        width: icloud_photo.width,
        height: icloud_photo.height,
        last_sync: Utc::now(),
        local_path: content_dir
            .join(photo_id)
            .join("original.jpg")
            .to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    fn create_test_photo() -> IndexedPhoto {
        IndexedPhoto {
            guid: "test_guid_123".to_string(),
            filename: "test_image.jpg".to_string(),
            caption: Some("Test Caption".to_string()),
            created_at: Utc::now(),
            checksum: "abcdef1234567890".to_string(),
            url: "https://example.com/test.jpg".to_string(),
            width: 1920,
            height: 1080,
            last_sync: Utc::now(),
            local_path: PathBuf::from("/content/photostream/test_photo/original.jpg"),
        }
    }
    
    #[test]
    fn test_new_index() {
        let index = PhotoIndex::new();
        assert!(index.photos.is_empty());
        assert!(index.last_updated <= Utc::now());
    }
    
    #[test]
    fn test_add_update_remove_photo() {
        let mut index = PhotoIndex::new();
        let photo = create_test_photo();
        
        // Add photo
        index.add_or_update_photo(photo.clone());
        assert_eq!(index.photo_count(), 1);
        
        // Get photo
        let retrieved = index.get_photo(&photo.guid).unwrap();
        assert_eq!(retrieved.filename, "test_image.jpg");
        
        // Update photo
        let mut updated_photo = photo.clone();
        updated_photo.caption = Some("Updated Caption".to_string());
        index.add_or_update_photo(updated_photo);
        assert_eq!(index.photo_count(), 1);
        
        let retrieved = index.get_photo(&photo.guid).unwrap();
        assert_eq!(retrieved.caption, Some("Updated Caption".to_string()));
        
        // Remove photo
        let removed = index.remove_photo(&photo.guid).unwrap();
        assert_eq!(removed.guid, photo.guid);
        assert_eq!(index.photo_count(), 0);
        assert!(index.get_photo(&photo.guid).is_none());
    }
    
    #[test]
    fn test_save_load_index() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("photos/index.yaml");
        
        // Create and save an index
        let mut index = PhotoIndex::new();
        let photo1 = create_test_photo();
        let mut photo2 = create_test_photo();
        photo2.guid = "test_guid_456".to_string();
        
        index.add_or_update_photo(photo1);
        index.add_or_update_photo(photo2);
        
        // Save to file
        index.save(&index_path)?;
        
        // Load from file
        let loaded_index = PhotoIndex::load(&index_path)?;
        
        // Verify content
        assert_eq!(loaded_index.photo_count(), 2);
        assert!(loaded_index.get_photo("test_guid_123").is_some());
        assert!(loaded_index.get_photo("test_guid_456").is_some());
        
        let photo = loaded_index.get_photo("test_guid_123").unwrap();
        assert_eq!(photo.filename, "test_image.jpg");
        
        Ok(())
    }
    
    #[test]
    fn test_load_nonexistent_creates_new() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("nonexistent_index.yaml");
        
        // Load from nonexistent file (should create new empty index)
        let index = PhotoIndex::load(&index_path)?;
        
        // Verify it's a new empty index
        assert_eq!(index.photo_count(), 0);
        
        Ok(())
    }
}