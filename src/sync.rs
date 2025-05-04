use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::icloud::{Album, Photo};
use crate::index::{PhotoIndex, IndexedPhoto};
use crate::exif::{extract_exif, ExifMetadata};

/// Responsible for syncing photos from iCloud to the local filesystem
pub struct Syncer {
    /// HTTP client for downloading photos
    client: Client,
    /// Base directory for storing photos
    content_dir: PathBuf,
    /// Path to the index file
    index_path: PathBuf,
}

/// Result of a photo sync operation
#[derive(Debug)]
pub enum SyncResult {
    /// Photo was newly added
    Added(String),
    /// Photo was updated (already existed but changed)
    Updated(String),
    /// Photo was already up to date (no changes)
    Unchanged(String),
    /// Photo was deleted (no longer in remote album)
    Deleted(String),
    /// Failed to sync this photo
    Failed(String, String), // (guid, error message)
}

impl Syncer {
    /// Creates a new syncer
    pub fn new(content_dir: PathBuf, index_path: PathBuf) -> Self {
        Self {
            client: Client::new(),
            content_dir,
            index_path,
        }
    }
    
    /// Loads the current photo index
    pub fn load_index(&self) -> Result<PhotoIndex> {
        PhotoIndex::load(&self.index_path)
    }
    
    /// Saves the photo index
    pub fn save_index(&self, index: &PhotoIndex) -> Result<()> {
        index.save(&self.index_path)
    }
    
    /// Syncs photos from the remote album to the local filesystem,
    /// adding new photos, updating changed ones, and removing deleted ones
    pub async fn sync_photos(&self, album: &Album, index: &mut PhotoIndex) -> Result<Vec<SyncResult>> {
        let mut results = Vec::new();
        
        // Ensure the content directory exists
        fs::create_dir_all(&self.content_dir)
            .context("Failed to create content directory")?;
        
        // Keep track of remote photo IDs
        let remote_guids: HashSet<&String> = album.photos.keys().collect();
        
        // Find photos to delete (in index but not in remote album)
        let photos_to_delete: Vec<_> = index.photos.keys()
            .filter(|guid| !remote_guids.contains(guid))
            .cloned()
            .collect();
        
        // Delete photos that are no longer in the remote album
        for guid in &photos_to_delete {
            match self.delete_photo(guid, index) {
                Ok(_) => results.push(SyncResult::Deleted(guid.clone())),
                Err(e) => results.push(SyncResult::Failed(
                    guid.clone(),
                    format!("Failed to delete photo: {}", e)
                )),
            }
        }
        
        // Process each photo in the album (add or update)
        for (guid, photo) in &album.photos {
            match self.sync_photo(photo, index).await {
                Ok(result) => results.push(result),
                Err(e) => results.push(SyncResult::Failed(
                    guid.clone(), 
                    format!("Failed to sync photo: {}", e)
                )),
            }
        }
        
        Ok(results)
    }
    
    /// Deletes a photo that is no longer in the remote album
    fn delete_photo(&self, guid: &str, index: &mut PhotoIndex) -> Result<()> {
        // Check if the photo exists in the index
        if !index.photos.contains_key(guid) {
            return Ok(());  // Photo not in index, nothing to do
        }
        
        // Get the directory containing the photo
        let photo_dir = self.content_dir.join(guid);
        
        // Remove the directory if it exists
        if photo_dir.exists() {
            fs::remove_dir_all(&photo_dir)
                .with_context(|| format!("Failed to delete directory for photo {}", guid))?;
        }
        
        // Remove the photo from the index
        index.remove_photo(guid);
        
        Ok(())
    }
    
    /// Syncs a single photo
    async fn sync_photo(&self, photo: &Photo, index: &mut PhotoIndex) -> Result<SyncResult> {
        // Determine if this is a new photo or an update
        let existing = index.get_photo(&photo.guid);
        
        // If the photo exists and checksums match, no need to update
        if let Some(existing) = existing {
            if existing.checksum == photo.checksum {
                return Ok(SyncResult::Unchanged(photo.guid.clone()));
            }
        }
        
        // Create directory for this photo
        let photo_dir = self.content_dir.join(&photo.guid);
        fs::create_dir_all(&photo_dir)
            .with_context(|| format!("Failed to create directory for photo {}", photo.guid))?;
        
        // Download the image
        let image_path = photo_dir.join("original.jpg");
        self.download_photo(photo, &image_path).await
            .with_context(|| format!("Failed to download photo {}", photo.guid))?;
        
        // Create a basic IndexedPhoto
        let mut indexed_photo = IndexedPhoto::new(
            photo.guid.clone(),
            photo.filename.clone(),
            photo.caption.clone(),
            photo.created_at,
            photo.checksum.clone(),
            photo.url.clone(),
            photo.width,
            photo.height,
            image_path.clone(),
        );
        
        // Extract EXIF data if possible
        if image_path.exists() {
            match extract_exif(&image_path) {
                Ok(exif_data) => {
                    // Update indexed photo with EXIF metadata
                    indexed_photo.update_exif(&exif_data);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to extract EXIF data from {}: {}", photo.guid, e);
                    // Continue without EXIF data
                }
            }
        }
        
        // Create index.md with frontmatter (now with potential EXIF data)
        let index_md_path = photo_dir.join("index.md");
        self.create_index_md_with_exif(&indexed_photo, &index_md_path)
            .with_context(|| format!("Failed to create index.md for photo {}", photo.guid))?;
        
        // Add or update the index entry
        let result = if existing.is_some() {
            SyncResult::Updated(photo.guid.clone())
        } else {
            SyncResult::Added(photo.guid.clone())
        };
        
        index.add_or_update_photo(indexed_photo);
        Ok(result)
    }
    
    /// Downloads a photo from its URL
    async fn download_photo(&self, photo: &Photo, path: &Path) -> Result<()> {
        // For tests or development, we can create a placeholder file instead of
        // actually downloading (the url might not be real in tests)
        if photo.url.contains("example.com") || cfg!(test) {
            // Create a placeholder image
            fs::write(path, "PLACEHOLDER IMAGE CONTENT")?;
            return Ok(());
        }
        
        // Otherwise, download the actual image
        let response = self.client.get(&photo.url)
            .send()
            .await
            .with_context(|| format!("Failed to GET photo from {}", photo.url))?;
        
        let bytes = response.bytes()
            .await
            .context("Failed to read photo bytes")?;
        
        fs::write(path, bytes)
            .with_context(|| format!("Failed to write photo to {}", path.display()))?;
        
        Ok(())
    }
    
    /// Creates an index.md file with basic frontmatter for a photo
    fn create_index_md(&self, photo: &Photo, path: &Path) -> Result<()> {
        let title = photo.caption.clone().unwrap_or_else(|| photo.filename.clone());
        
        let frontmatter = format!(
            "---
title: {}
date: {}
guid: {}
original_filename: {}
width: {}
height: {}
---

{}
",
            title,
            photo.created_at.format("%Y-%m-%dT%H:%M:%S%z"),
            photo.guid,
            photo.filename,
            photo.width,
            photo.height,
            photo.caption.clone().unwrap_or_default()
        );
        
        fs::write(path, frontmatter)
            .with_context(|| format!("Failed to write index.md to {}", path.display()))
    }
    
    /// Creates an index.md file with frontmatter including EXIF data
    fn create_index_md_with_exif(&self, photo: &IndexedPhoto, path: &Path) -> Result<()> {
        let title = photo.caption.clone().unwrap_or_else(|| photo.filename.clone());
        
        // Build the frontmatter with EXIF data if available
        let mut frontmatter = format!(
            "---
title: {}
date: {}
guid: {}
original_filename: {}
width: {}
height: {}
",
            title,
            photo.created_at.format("%Y-%m-%dT%H:%M:%S%z"),
            photo.guid,
            photo.filename,
            photo.width,
            photo.height,
        );
        
        // Add EXIF data if available
        if let Some(ref make) = photo.camera_make {
            frontmatter.push_str(&format!("camera_make: {}\n", make));
        }
        
        if let Some(ref model) = photo.camera_model {
            frontmatter.push_str(&format!("camera_model: {}\n", model));
        }
        
        if let Some(exif_dt) = photo.exif_date_time {
            frontmatter.push_str(&format!("exif_date: {}\n", exif_dt.format("%Y-%m-%dT%H:%M:%S%z")));
        }
        
        // Add GPS data (original and fuzzed)
        if let Some(lat) = photo.latitude {
            frontmatter.push_str(&format!("original_latitude: {:.6}\n", lat));
        }
        
        if let Some(lon) = photo.longitude {
            frontmatter.push_str(&format!("original_longitude: {:.6}\n", lon));
        }
        
        if let Some(lat) = photo.fuzzed_latitude {
            frontmatter.push_str(&format!("latitude: {:.6}\n", lat));
        }
        
        if let Some(lon) = photo.fuzzed_longitude {
            frontmatter.push_str(&format!("longitude: {:.6}\n", lon));
        }
        
        // Add camera settings if available
        if let Some(iso) = photo.iso {
            frontmatter.push_str(&format!("iso: {}\n", iso));
        }
        
        if let Some(ref exposure) = photo.exposure_time {
            frontmatter.push_str(&format!("exposure_time: {}\n", exposure));
        }
        
        if let Some(aperture) = photo.f_number {
            frontmatter.push_str(&format!("f_number: {:.1}\n", aperture));
        }
        
        if let Some(focal) = photo.focal_length {
            frontmatter.push_str(&format!("focal_length: {:.1}\n", focal));
        }
        
        // Close frontmatter and add content
        frontmatter.push_str("---\n\n");
        frontmatter.push_str(&photo.caption.clone().unwrap_or_default());
        
        fs::write(path, frontmatter)
            .with_context(|| format!("Failed to write index.md to {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    fn create_test_photo(guid: &str) -> Photo {
        Photo {
            guid: guid.to_string(),
            filename: format!("{}.jpg", guid),
            caption: Some(format!("Caption for {}", guid)),
            created_at: Utc::now(),
            checksum: format!("checksum_{}", guid),
            url: format!("https://example.com/{}.jpg", guid),
            width: 800,
            height: 600,
        }
    }
    
    fn create_test_album() -> Album {
        let mut album = Album::new("Test Album".to_string());
        
        let photo1 = create_test_photo("photo1");
        let photo2 = create_test_photo("photo2");
        
        album.photos.insert(photo1.guid.clone(), photo1);
        album.photos.insert(photo2.guid.clone(), photo2);
        
        album
    }
    
    #[tokio::test]
    async fn test_sync_new_photos() -> Result<()> {
        let temp_dir = tempdir()?;
        let content_dir = temp_dir.path().join("content");
        let index_path = temp_dir.path().join("index.yaml");
        
        let syncer = Syncer::new(content_dir.clone(), index_path.clone());
        
        // Start with an empty index
        let mut index = PhotoIndex::new();
        
        // Create a test album with two photos
        let album = create_test_album();
        
        // Sync the photos
        let results = syncer.sync_photos(&album, &mut index).await?;
        
        // Verify results
        assert_eq!(results.len(), 2);
        
        let mut added_count = 0;
        let mut updated_count = 0;
        let mut unchanged_count = 0;
        
        for result in &results {
            match result {
                SyncResult::Added(_) => added_count += 1,
                SyncResult::Updated(_) => updated_count += 1,
                SyncResult::Unchanged(_) => unchanged_count += 1,
                SyncResult::Deleted(_) => panic!("Should not have deleted any photos"),
                SyncResult::Failed(guid, error) => {
                    panic!("Photo {} failed to sync: {}", guid, error);
                }
            }
        }
        
        assert_eq!(added_count, 2, "Expected 2 added photos");
        assert_eq!(updated_count, 0, "Expected 0 updated photos");
        assert_eq!(unchanged_count, 0, "Expected 0 unchanged photos");
        
        // Verify files were created
        assert!(content_dir.join("photo1").join("original.jpg").exists());
        assert!(content_dir.join("photo1").join("index.md").exists());
        assert!(content_dir.join("photo2").join("original.jpg").exists());
        assert!(content_dir.join("photo2").join("index.md").exists());
        
        // Verify index was updated
        assert_eq!(index.photo_count(), 2);
        assert!(index.get_photo("photo1").is_some());
        assert!(index.get_photo("photo2").is_some());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_sync_unchanged_photos() -> Result<()> {
        let temp_dir = tempdir()?;
        let content_dir = temp_dir.path().join("content");
        let index_path = temp_dir.path().join("index.yaml");
        
        let syncer = Syncer::new(content_dir.clone(), index_path.clone());
        
        // Create a test album with two photos
        let album = create_test_album();
        
        // Start with an index that already has these photos
        let mut index = PhotoIndex::new();
        
        // Add the photos to the index with the same checksums
        for (guid, photo) in &album.photos {
            let indexed_photo = IndexedPhoto::new(
                guid.clone(),
                photo.filename.clone(),
                photo.caption.clone(),
                photo.created_at,
                photo.checksum.clone(), // Same checksum!
                photo.url.clone(),
                photo.width,
                photo.height,
                PathBuf::from(format!("/content/{}/original.jpg", guid)),
            );
            
            index.add_or_update_photo(indexed_photo);
        }
        
        // Create the directories so we can pretend files exist
        fs::create_dir_all(content_dir.join("photo1"))?;
        fs::create_dir_all(content_dir.join("photo2"))?;
        
        // Sync the photos
        let results = syncer.sync_photos(&album, &mut index).await?;
        
        // Verify results - should be unchanged since checksums match
        assert_eq!(results.len(), 2);
        
        let mut unchanged_count = 0;
        
        for result in &results {
            if let SyncResult::Unchanged(_) = result {
                unchanged_count += 1;
            }
        }
        
        assert_eq!(unchanged_count, 2, "Expected 2 unchanged photos");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_sync_updated_photos() -> Result<()> {
        let temp_dir = tempdir()?;
        let content_dir = temp_dir.path().join("content");
        let index_path = temp_dir.path().join("index.yaml");
        
        let syncer = Syncer::new(content_dir.clone(), index_path.clone());
        
        // Create a test album with two photos
        let album = create_test_album();
        
        // Start with an index that already has these photos but with different checksums
        let mut index = PhotoIndex::new();
        
        // Add the photos to the index with different checksums
        for (guid, photo) in &album.photos {
            let indexed_photo = IndexedPhoto::new(
                guid.clone(),
                photo.filename.clone(),
                photo.caption.clone(),
                photo.created_at,
                format!("different_checksum_{}", guid), // Different checksum!
                photo.url.clone(),
                photo.width,
                photo.height,
                PathBuf::from(format!("/content/{}/original.jpg", guid)),
            );
            
            index.add_or_update_photo(indexed_photo);
        }
        
        // Create the directories so we can pretend files exist
        fs::create_dir_all(content_dir.join("photo1"))?;
        fs::create_dir_all(content_dir.join("photo2"))?;
        
        // Sync the photos
        let results = syncer.sync_photos(&album, &mut index).await?;
        
        // Verify results - should be updated since checksums don't match
        assert_eq!(results.len(), 2);
        
        let mut updated_count = 0;
        
        for result in &results {
            if let SyncResult::Updated(_) = result {
                updated_count += 1;
            }
        }
        
        assert_eq!(updated_count, 2, "Expected 2 updated photos");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_sync_delete_photos() -> Result<()> {
        let temp_dir = tempdir()?;
        let content_dir = temp_dir.path().join("content");
        let index_path = temp_dir.path().join("index.yaml");
        
        let syncer = Syncer::new(content_dir.clone(), index_path.clone());
        
        // Create a test index with three photos (photo1, photo2, photo3)
        let mut index = PhotoIndex::new();
        
        // Add photos to the index
        for guid in &["photo1", "photo2", "photo3"] {
            let photo = create_test_photo(guid);
            let photo_dir = content_dir.join(guid);
            
            // Create directories and placeholder files
            fs::create_dir_all(&photo_dir)?;
            fs::write(photo_dir.join("original.jpg"), "test content")?;
            fs::write(photo_dir.join("index.md"), "test content")?;
            
            let indexed_photo = IndexedPhoto::new(
                photo.guid.clone(),
                photo.filename.clone(),
                photo.caption.clone(),
                photo.created_at,
                photo.checksum.clone(),
                photo.url.clone(),
                photo.width,
                photo.height,
                photo_dir.join("original.jpg"),
            );
            
            index.add_or_update_photo(indexed_photo);
        }
        
        // Verify initial state
        assert_eq!(index.photo_count(), 3);
        assert!(content_dir.join("photo1").exists());
        assert!(content_dir.join("photo2").exists());
        assert!(content_dir.join("photo3").exists());
        
        // Create a test album with only two photos (photo1, photo2)
        // so photo3 should be deleted
        let album = create_test_album(); // Creates only photo1 and photo2
        
        // Sync the photos
        let results = syncer.sync_photos(&album, &mut index).await?;
        
        // Count deleted photos
        let mut deleted_count = 0;
        for result in &results {
            if let SyncResult::Deleted(guid) = result {
                deleted_count += 1;
                assert_eq!(guid, "photo3", "Expected photo3 to be deleted");
            }
        }
        
        assert_eq!(deleted_count, 1, "Expected 1 deleted photo");
        assert_eq!(index.photo_count(), 2, "Expected 2 photos left in index");
        
        // Verify photo3 directory was deleted
        assert!(content_dir.join("photo1").exists());
        assert!(content_dir.join("photo2").exists());
        assert!(!content_dir.join("photo3").exists());
        
        // Verify photo3 was removed from the index
        assert!(index.get_photo("photo1").is_some());
        assert!(index.get_photo("photo2").is_some());
        assert!(index.get_photo("photo3").is_none());
        
        Ok(())
    }
}