//! Local photo index management for icloud2hugo.
//!
//! This module handles the storage and retrieval of local photo metadata
//! in a YAML-based index file. It defines the `PhotoIndex` struct for the overall
//! collection and the `IndexedPhoto` struct to represent individual photo metadata.
//!
//! The index tracks various information about each photo, including:
//! - Basic metadata (filename, dimensions, etc.)
//! - EXIF data (camera info, date/time, GPS coordinates)
//! - Location information from reverse geocoding
//! - Gallery information for organizing photos into collections
//!
//! This allows the application to efficiently determine which photos need
//! to be added, updated, or removed during synchronization.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::geocode::Location;

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

    // EXIF metadata
    /// Make of the camera used (e.g., "Apple")
    pub camera_make: Option<String>,
    /// Model of the camera used (e.g., "iPhone 15 Pro")
    pub camera_model: Option<String>,
    /// Precise date/time when the photo was taken (from EXIF)
    pub exif_date_time: Option<DateTime<Utc>>,
    /// Original latitude from EXIF data
    pub latitude: Option<f64>,
    /// Original longitude from EXIF data
    pub longitude: Option<f64>,
    /// Fuzzed latitude for privacy
    pub fuzzed_latitude: Option<f64>,
    /// Fuzzed longitude for privacy
    pub fuzzed_longitude: Option<f64>,
    /// ISO speed rating
    pub iso: Option<u32>,
    /// Exposure time
    pub exposure_time: Option<String>,
    /// F-number / aperture
    pub f_number: Option<f32>,
    /// Focal length in mm
    pub focal_length: Option<f32>,

    // Location information from geocoding
    /// Formatted location address (e.g., "Chicago, IL, USA")
    pub location: Option<Location>,
}

/// Represents a gallery collection of photos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gallery {
    /// Unique identifier for the gallery
    pub id: String,
    /// Display name for the gallery
    pub name: String,
    /// URL-friendly name for the gallery directory
    pub slug: String,
    /// Optional description of the gallery
    pub description: Option<String>,
    /// List of photo GUIDs included in this gallery
    pub photos: Vec<String>,
    /// When the gallery was created
    pub created_at: DateTime<Utc>,
    /// When the gallery was last updated
    pub updated_at: DateTime<Utc>,
}

impl Gallery {
    /// Creates a new gallery with the given name
    pub fn new(id: String, name: String, slug: String, description: Option<String>) -> Self {
        Self {
            id,
            name,
            slug,
            description,
            photos: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Adds a photo to the gallery
    pub fn add_photo(&mut self, guid: String) {
        if !self.photos.contains(&guid) {
            self.photos.push(guid);
            self.updated_at = Utc::now();
        }
    }

    /// Removes a photo from the gallery
    pub fn remove_photo(&mut self, guid: &str) {
        if let Some(index) = self.photos.iter().position(|p| p == guid) {
            self.photos.remove(index);
            self.updated_at = Utc::now();
        }
    }
}

/// Represents our local database of photos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoIndex {
    /// When the index was last updated
    pub last_updated: DateTime<Utc>,
    /// Map of photo GUIDs to indexed photos
    pub photos: HashMap<String, IndexedPhoto>,
    /// Map of gallery IDs to galleries
    pub galleries: HashMap<String, Gallery>,
}

impl IndexedPhoto {
    /// Creates a new photo index entry with minimal data
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid: String,
        filename: String,
        caption: Option<String>,
        created_at: DateTime<Utc>,
        checksum: String,
        url: String,
        width: u32,
        height: u32,
        local_path: PathBuf,
    ) -> Self {
        Self {
            guid,
            filename,
            caption,
            created_at,
            checksum,
            url,
            width,
            height,
            last_sync: Utc::now(),
            local_path,
            camera_make: None,
            camera_model: None,
            exif_date_time: None,
            latitude: None,
            longitude: None,
            fuzzed_latitude: None,
            fuzzed_longitude: None,
            iso: None,
            exposure_time: None,
            f_number: None,
            focal_length: None,
            location: None,
        }
    }

    /// Update this photo with EXIF metadata
    pub fn update_exif(&mut self, exif: &crate::exif::ExifMetadata) {
        self.camera_make = exif.camera_make.clone();
        self.camera_model = exif.camera_model.clone();
        self.exif_date_time = exif.date_time;
        self.latitude = exif.latitude;
        self.longitude = exif.longitude;
        self.fuzzed_latitude = exif.fuzzed_latitude;
        self.fuzzed_longitude = exif.fuzzed_longitude;
        self.iso = exif.iso;
        self.exposure_time = exif.exposure_time.clone();
        self.f_number = exif.f_number;
        self.focal_length = exif.focal_length;
    }

    /// Update this photo with location data from geocoding
    pub fn update_location(&mut self, location: crate::geocode::Location) {
        self.location = Some(location);
    }
}

impl PhotoIndex {
    /// Creates a new empty photo index
    pub fn new() -> Self {
        Self {
            last_updated: Utc::now(),
            photos: HashMap::new(),
            galleries: HashMap::new(),
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
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create directory for {}", path.display())
                })?;
            }
        }

        // Serialize and write to file
        let yaml =
            serde_yaml::to_string(self).with_context(|| "Failed to serialize index to YAML")?;

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
        // Remove photo from all galleries
        for gallery in self.galleries.values_mut() {
            gallery.remove_photo(guid);
        }

        // Remove from photos collection
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

    /// Add a new gallery or update an existing one
    pub fn add_or_update_gallery(&mut self, gallery: Gallery) {
        self.galleries.insert(gallery.id.clone(), gallery);
        self.last_updated = Utc::now();
    }

    /// Remove a gallery from the index
    #[allow(dead_code)]
    pub fn remove_gallery(&mut self, id: &str) -> Option<Gallery> {
        let result = self.galleries.remove(id);
        if result.is_some() {
            self.last_updated = Utc::now();
        }
        result
    }

    /// Get a gallery by ID
    pub fn get_gallery(&self, id: &str) -> Option<&Gallery> {
        self.galleries.get(id)
    }

    /// Number of galleries in the index
    pub fn gallery_count(&self) -> usize {
        self.galleries.len()
    }

    /// Get all photos in a gallery
    pub fn get_gallery_photos(&self, gallery_id: &str) -> Vec<&IndexedPhoto> {
        if let Some(gallery) = self.galleries.get(gallery_id) {
            gallery
                .photos
                .iter()
                .filter_map(|guid| self.photos.get(guid))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for PhotoIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts an iCloud photo to our indexed photo format
#[allow(dead_code)]
pub fn convert_to_indexed_photo(
    icloud_photo: &crate::icloud::Photo,
    content_dir: &Path,
    photo_id: &str,
) -> IndexedPhoto {
    IndexedPhoto::new(
        icloud_photo.guid.clone(),
        icloud_photo.filename.clone(),
        icloud_photo.caption.clone(),
        icloud_photo.created_at,
        icloud_photo.checksum.clone(),
        icloud_photo.url.clone(),
        icloud_photo.width,
        icloud_photo.height,
        content_dir
            .join(photo_id)
            .join("original.jpg")
            .to_path_buf(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_photo() -> IndexedPhoto {
        IndexedPhoto::new(
            "test_guid_123".to_string(),
            "test_image.jpg".to_string(),
            Some("Test Caption".to_string()),
            Utc::now(),
            "abcdef1234567890".to_string(),
            "https://example.com/test.jpg".to_string(),
            1920,
            1080,
            PathBuf::from("/content/photostream/test_photo/original.jpg"),
        )
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
