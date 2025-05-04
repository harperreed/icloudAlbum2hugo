use anyhow::Result;
use chrono::Utc;
use crate::icloud::{Album, Photo};

/// Creates a mock album for testing
pub fn create_mock_album() -> Result<Album> {
    let mut album = Album::new("Mock Test Album".to_string());
    
    // Add a few test photos
    album.photos.insert("mock1".to_string(), Photo {
        guid: "mock1".to_string(),
        filename: "mock1.jpg".to_string(),
        caption: Some("Mock Photo 1".to_string()),
        created_at: Utc::now(),
        checksum: "mock_checksum_1".to_string(),
        url: "https://example.com/mock1.jpg".to_string(),
        width: 1200,
        height: 800,
    });
    
    album.photos.insert("mock2".to_string(), Photo {
        guid: "mock2".to_string(),
        filename: "mock2.jpg".to_string(),
        caption: None,
        created_at: Utc::now(),
        checksum: "mock_checksum_2".to_string(),
        url: "https://example.com/mock2.jpg".to_string(),
        width: 1920,
        height: 1080,
    });
    
    album.photos.insert("mock3".to_string(), Photo {
        guid: "mock3".to_string(),
        filename: "photo_with_no_caption.jpg".to_string(),
        caption: None,
        created_at: Utc::now(),
        checksum: "mock_checksum_3".to_string(),
        url: "https://example.com/mock3.jpg".to_string(),
        width: 800,
        height: 600,
    });
    
    Ok(album)
}

/// Simulates fetching an album from a URL.
/// For test URLs, returns a mock album.
pub async fn mock_fetch_album(album_url: &str) -> Result<Album> {
    if album_url.contains("test") || 
       album_url.contains("example") || 
       album_url.contains("custom") ||
       cfg!(test) {
        return create_mock_album();
    }
    
    // In a real implementation, this would call the actual fetch_album
    Err(anyhow::anyhow!("Only test URLs are supported in mock mode"))
}