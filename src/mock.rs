use crate::icloud::{Album, Photo};
use anyhow::Result;
use chrono::Utc;

// Test data constants for mock content
const MOCK_ALBUM_NAME: &str = "Mock Test Album";
const MOCK_URL_PREFIX: &str = "https://test.example/";
const MOCK_CHECKSUM_PREFIX: &str = "mock_checksum_";

/// Creates a mock album for testing
pub fn create_mock_album() -> Result<Album> {
    let mut album = Album::new(MOCK_ALBUM_NAME.to_string());

    // Add a few test photos
    album.photos.insert(
        "mock1".to_string(),
        Photo {
            guid: "mock1".to_string(),
            filename: "mock1.jpg".to_string(),
            caption: Some("Mock Photo 1".to_string()),
            created_at: Utc::now(),
            checksum: format!("{}{}", MOCK_CHECKSUM_PREFIX, "1"),
            url: format!("{}mock1.jpg", MOCK_URL_PREFIX),
            width: 1200,
            height: 800,
        },
    );

    album.photos.insert(
        "mock2".to_string(),
        Photo {
            guid: "mock2".to_string(),
            filename: "mock2.jpg".to_string(),
            caption: None,
            created_at: Utc::now(),
            checksum: format!("{}{}", MOCK_CHECKSUM_PREFIX, "2"),
            url: format!("{}mock2.jpg", MOCK_URL_PREFIX),
            width: 1920,
            height: 1080,
        },
    );

    album.photos.insert(
        "mock3".to_string(),
        Photo {
            guid: "mock3".to_string(),
            filename: "photo_with_no_caption.jpg".to_string(),
            caption: None,
            created_at: Utc::now(),
            checksum: format!("{}{}", MOCK_CHECKSUM_PREFIX, "3"),
            url: format!("{}mock3.jpg", MOCK_URL_PREFIX),
            width: 800,
            height: 600,
        },
    );

    Ok(album)
}

// Constants for test URL identification
#[allow(dead_code)]
const TEST_URL_INDICATORS: [&str; 3] = ["test", "example", "custom"];

/// Simulates fetching an album from a URL.
/// For test URLs, returns a mock album.
#[allow(dead_code)]
pub async fn mock_fetch_album(album_url: &str) -> Result<Album> {
    // Check if any of our test indicators are in the URL, or if we're in test mode
    let is_test_url = TEST_URL_INDICATORS
        .iter()
        .any(|indicator| album_url.contains(indicator))
        || cfg!(test);

    if is_test_url {
        return create_mock_album();
    }

    // In a real implementation, this would call the actual fetch_album
    Err(anyhow::anyhow!("Only test URLs are supported in mock mode"))
}
