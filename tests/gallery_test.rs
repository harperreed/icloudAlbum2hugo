//! Tests for the gallery functionality
//!
//! These tests verify the gallery functionality including:
//! - Gallery data structures
//! - Multi-output configuration
//! - Gallery syncing process

use icloudAlbum2hugo::config::{Config, OutputConfig, OutputType, PrivacyConfig};
use icloudAlbum2hugo::gallery::GallerySyncer;
use icloudAlbum2hugo::icloud::{Album, Photo};
use icloudAlbum2hugo::index::{Gallery, PhotoIndex};

use chrono::Utc;
use tempfile::tempdir;

/// Create a test photo with the given ID
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
        mime_type: "image/jpeg".to_string(),
    }
}

/// Create a test album with the given number of photos
fn create_test_album(name: &str, photo_count: usize) -> Album {
    let mut album = Album::new(name.to_string());

    for i in 0..photo_count {
        let guid = format!("photo{}", i + 1);
        let photo = create_test_photo(&guid);
        album.photos.insert(guid, photo);
    }

    album
}

/// Test PhotoIndex gallery operations
#[test]
fn test_photo_index_gallery_operations() {
    // Create a new photo index
    let mut index = PhotoIndex::new();

    // Create a test gallery
    let gallery = Gallery::new(
        "gallery1".to_string(),
        "Test Gallery".to_string(),
        "test-gallery".to_string(),
        Some("Test Gallery Description".to_string()),
    );

    // Add the gallery to the index
    index.add_or_update_gallery(gallery);

    // Verify gallery count
    assert_eq!(index.gallery_count(), 1);

    // Verify gallery retrieval
    let retrieved = index.get_gallery("gallery1").unwrap();
    assert_eq!(retrieved.name, "Test Gallery");
    assert_eq!(retrieved.slug, "test-gallery");
    assert_eq!(
        retrieved.description,
        Some("Test Gallery Description".to_string())
    );

    // Add a photo to the gallery
    let mut updated_gallery = retrieved.clone();
    updated_gallery.add_photo("photo1".to_string());
    index.add_or_update_gallery(updated_gallery);

    // Verify photo addition
    let updated = index.get_gallery("gallery1").unwrap();
    assert_eq!(updated.photos.len(), 1);
    assert_eq!(updated.photos[0], "photo1");

    // Remove the gallery
    let removed = index.remove_gallery("gallery1");
    assert!(removed.is_some());
    assert_eq!(index.gallery_count(), 0);
}

/// Test multi-output configuration
#[test]
fn test_multi_output_configuration() {
    // Create a test configuration
    let mut config = Config::default();

    // Add a gallery output
    let gallery_output = OutputConfig {
        output_type: OutputType::Gallery,
        album_url: "https://example.com/gallery".to_string(),
        out_dir: "content/gallery".to_string(),
        data_file: "data/gallery.yaml".to_string(),
        name: Some("Test Gallery".to_string()),
        description: Some("Test Description".to_string()),
        ..Default::default()
    };

    config.outputs.push(gallery_output);

    // Verify output count
    assert_eq!(config.outputs.len(), 2);

    // Verify enabled outputs
    let enabled = config.enabled_outputs();
    assert_eq!(enabled.len(), 2);

    // Verify filtering by name
    let filtered = config.get_outputs_by_name(&["Test Gallery".to_string()]);
    assert_eq!(filtered.len(), 1);
    assert!(matches!(filtered[0].output_type, OutputType::Gallery));

    // Disable an output
    config.outputs[0].enabled = false;

    // Verify enabled outputs again
    let enabled = config.enabled_outputs();
    assert_eq!(enabled.len(), 1);
    assert!(matches!(enabled[0].output_type, OutputType::Gallery));
}

/// Test gallery syncing
#[tokio::test]
async fn test_gallery_syncing() -> anyhow::Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let content_dir = temp_dir.path().join("content");
    let index_path = temp_dir.path().join("index.yaml");

    // Create a test album
    let album = create_test_album("Test Album", 3);

    // Create a gallery syncer
    let gallery_syncer = GallerySyncer::new(
        content_dir.clone(),
        Some("Test Gallery".to_string()),
        Some("Test Description".to_string()),
        index_path.clone(),
        PrivacyConfig::default(),
    );

    // Create a photo index
    let mut index = PhotoIndex::new();

    // Sync the gallery
    let results = gallery_syncer.sync_gallery(&album, &mut index).await?;

    // Verify results
    assert_eq!(results.len(), 3);

    // Check result types (all should be Added)
    let mut added_count = 0;
    for result in &results {
        if let icloudAlbum2hugo::sync::SyncResult::Added(_) = result {
            added_count += 1;
        }
    }
    assert_eq!(added_count, 3);

    // Verify files were created
    assert!(content_dir.join("photo1.jpg").exists());
    assert!(content_dir.join("photo2.jpg").exists());
    assert!(content_dir.join("photo3.jpg").exists());
    assert!(content_dir.join("index.md").exists());

    // Verify index data
    assert_eq!(index.photo_count(), 3);
    assert_eq!(index.gallery_count(), 1);

    let gallery = index.galleries.values().next().unwrap();
    assert_eq!(gallery.photos.len(), 3);

    // Test updating the gallery (add a new photo)
    let mut updated_album = album.clone();
    let new_photo = create_test_photo("photo4");
    updated_album.photos.insert("photo4".to_string(), new_photo);

    // Sync again
    let update_results = gallery_syncer
        .sync_gallery(&updated_album, &mut index)
        .await?;

    // Verify updated results
    assert_eq!(update_results.len(), 4); // 3 unchanged + 1 added

    // Check that the new photo was added
    assert!(content_dir.join("photo4.jpg").exists());

    // Verify updated index data
    assert_eq!(index.photo_count(), 4);

    let updated_gallery = index.galleries.values().next().unwrap();
    assert_eq!(updated_gallery.photos.len(), 4);

    Ok(())
}

/// Test legacy configuration handling
#[test]
fn test_legacy_config_handling() -> anyhow::Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.yaml");

    // Create a legacy format config
    let legacy_yaml = r#"
album_url: https://www.icloud.com/sharedalbum/legacy_token
out_dir: content/legacy
data_file: data/legacy.yaml
fuzz_meters: 50.0
"#;

    // Write the legacy config to file
    std::fs::write(&config_path, legacy_yaml)?;

    // Load the config
    let config = Config::load_from_file(&config_path)?;

    // Verify conversion
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

/// Test gallery with privacy settings
#[tokio::test]
async fn test_gallery_with_privacy_settings() -> anyhow::Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let content_dir = temp_dir.path().join("content");
    let index_path = temp_dir.path().join("index.yaml");

    // Create a test album
    let album = create_test_album("Privacy Test Album", 2);

    // Create privacy config with some settings enabled
    let privacy_config = PrivacyConfig {
        nofeed: true,
        uuid_slug: true,
        ..Default::default()
    };

    // Create a gallery syncer with privacy settings
    let gallery_syncer = GallerySyncer::new(
        content_dir.clone(),
        Some("Privacy Gallery".to_string()),
        Some("Test gallery with privacy".to_string()),
        index_path.clone(),
        privacy_config,
    );

    // Create a photo index
    let mut index = PhotoIndex::new();

    // Sync the gallery
    let results = gallery_syncer.sync_gallery(&album, &mut index).await?;

    // Verify results
    assert_eq!(results.len(), 2);

    // Verify files were created
    assert!(content_dir.join("photo1.jpg").exists());
    assert!(content_dir.join("photo2.jpg").exists());
    assert!(content_dir.join("index.md").exists());

    // Read the generated frontmatter
    let content = std::fs::read_to_string(content_dir.join("index.md"))?;

    // Verify privacy settings are in frontmatter
    assert!(content.contains("nofeed: true"));
    assert!(content.contains("uuid: "));
    assert!(content.contains("slug: "));

    // Verify that disabled privacy settings are not present
    assert!(!content.contains("noindex: true"));
    assert!(!content.contains("unlisted: true"));

    // Verify gallery has UUID
    let gallery = index.galleries.values().next().unwrap();
    assert!(!gallery.uuid.is_empty());
    assert!(gallery.uuid.len() > 10);

    Ok(())
}
