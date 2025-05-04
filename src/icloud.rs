//! # iCloud API Module
//!
//! This module handles interaction with iCloud shared albums, including URL parsing,
//! token extraction, album data fetching, and conversion to our internal data structures.
//!
//! ## Feature Overview
//!
//! - Supports multiple iCloud shared album URL formats
//! - Extracts tokens from iCloud URLs for API access
//! - Fetches album data including metadata and photos
//! - Finds the best available photo derivatives for downloading
//! - Processes photo metadata (dates, captions, etc.)
//! - Provides robust error handling and detailed logging
//!
//! ## Primary Functions
//!
//! - `fetch_album`: Main entry point for fetching an album by URL
//! - `extract_token`: Extracts the access token from an iCloud URL
//! - `find_best_derivative`: Selects the optimal photo resolution
//!
//! ## Error Handling
//!
//! The module defines a custom `ICloudError` type with variants for different
//! error scenarios (invalid URLs, network errors, etc.) and implements proper
//! error context and logging.

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use url::Url;

/// Represents a single photo in an album with all necessary metadata
///
/// This struct contains all the information needed to identify, download,
/// and display a photo from an iCloud shared album, including:
/// - Unique identifiers (GUID)
/// - Photo metadata (dimensions, creation date)
/// - Display information (caption)
/// - Download URL and verification (checksum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    /// Unique identifier for the photo
    pub guid: String,

    /// Filename to use when saving the photo locally
    pub filename: String,

    /// Optional caption/description for the photo
    pub caption: Option<String>,

    /// Creation date of the photo in UTC
    pub created_at: DateTime<Utc>,

    /// Checksum for verifying photo integrity
    pub checksum: String,

    /// URL for downloading the full-resolution photo
    pub url: String,

    /// Width of the photo in pixels
    pub width: u32,

    /// Height of the photo in pixels
    pub height: u32,
    
    /// MIME type of the photo (e.g., "image/jpeg", "image/png")
    #[serde(default = "default_mime_type")]
    pub mime_type: String,
}

/// Default MIME type for backward compatibility
fn default_mime_type() -> String {
    "image/jpeg".to_string()
}

/// Represents a collection of photos in an album
///
/// An album is a container for photos, with a name and a mapping of
/// photo GUIDs to Photo objects. This allows for efficient lookup
/// by GUID when comparing local and remote photos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// Display name of the album
    pub name: String,

    /// Map of photo GUIDs to Photo objects
    pub photos: HashMap<String, Photo>,
}

impl Album {
    /// Creates a new empty album with the given name
    ///
    /// # Arguments
    ///
    /// * `name` - The display name for the album
    ///
    /// # Returns
    ///
    /// A new empty Album instance
    pub fn new(name: String) -> Self {
        Self {
            name,
            photos: HashMap::new(),
        }
    }

    /// Returns the number of photos in the album
    ///
    /// # Returns
    ///
    /// The count of photos in the album
    #[allow(dead_code)]
    pub fn photo_count(&self) -> usize {
        self.photos.len()
    }
}

/// Represents different formats of iCloud shared album URLs
#[derive(Debug)]
enum ICloudUrlFormat {
    /// Standard format: https://www.icloud.com/sharedalbum/#B0abCdEfGhIj
    Standard,
    /// Web format with additional parameters: https://www.icloud.com/sharedalbum/#B0123456789?param=value
    WebWithParams,
    /// Invitation format: https://share.icloud.com/photos/abc0defGHIjklMNO
    Invitation,
    /// Unknown format
    Unknown,
}

/// Determines the format of the provided iCloud URL
fn determine_url_format(url_str: &str) -> ICloudUrlFormat {
    if url_str.contains("icloud.com/sharedalbum/#B") {
        // Check if there are query parameters
        if url_str.contains("?") {
            ICloudUrlFormat::WebWithParams
        } else {
            ICloudUrlFormat::Standard
        }
    } else if url_str.contains("share.icloud.com/photos/") {
        ICloudUrlFormat::Invitation
    } else {
        ICloudUrlFormat::Unknown
    }
}

/// Extracts the token from an iCloud shared album URL
///
/// Supports multiple URL formats:
/// - Standard: https://www.icloud.com/sharedalbum/#B0abCdEfGhIj
/// - With query parameters: https://www.icloud.com/sharedalbum/#B0123456789?param=value
/// - Invitation links: https://share.icloud.com/photos/abc0defGHIjklMNO
///
/// Returns the token as a String or an error if the URL format is invalid or unsupported
fn extract_token(album_url: &str) -> Result<String> {
    // First, determine the URL format
    let format = determine_url_format(album_url);

    match format {
        ICloudUrlFormat::Standard | ICloudUrlFormat::WebWithParams => {
            // Validate and parse the URL
            let url = ICloudError::context(
                Url::parse(album_url),
                format!("Invalid iCloud shared album URL: {}", album_url),
            )?;

            // Extract the token (shared album ID) from the URL fragment
            url.fragment()
                .and_then(|fragment| {
                    // In case the fragment contains query parameters (like #B0123456789?param=value)
                    let token_part = if fragment.contains('?') {
                        fragment.split('?').next()
                    } else {
                        Some(fragment)
                    };

                    // Validate that the token starts with 'B' which is typical for iCloud tokens
                    token_part
                        .filter(|t| t.starts_with("B"))
                        .map(|t| t.to_string())
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(ICloudError::InvalidToken(
                        "Missing or invalid token in fragment".to_string()
                    ))
                })
        }
        ICloudUrlFormat::Invitation => {
            // For invitation URLs like https://share.icloud.com/photos/abc0defGHIjklMNO
            // Extract the token from the path
            let url = ICloudError::context(
                Url::parse(album_url),
                format!("Invalid iCloud shared album invitation URL: {}", album_url),
            )?;

            // Use path_segments() which returns an iterator of segments
            let mut segments = url.path_segments().ok_or_else(|| {
                ICloudError::InvalidUrl("Invalid URL path: cannot be base".to_string())
            })?;

            // Find "photos" segment and get the next segment as the token
            if let Some("photos") = segments.next() {
                if let Some(token) = segments.next() {
                    return Ok(token.to_string());
                }
            }

            // If we didn't find a valid token after "photos"
            Err(anyhow::anyhow!(ICloudError::InvalidToken(
                "Unable to extract token from invitation URL path".to_string()
            )))
        }
        ICloudUrlFormat::Unknown => Err(anyhow::anyhow!(ICloudError::InvalidUrl(format!(
            "Unsupported iCloud URL format: {}",
            album_url
        )))),
    }
}

/// Custom error type for iCloud fetching operations
///
/// This enum provides specific error variants for different failure scenarios
/// when interacting with iCloud shared albums. Each variant includes a descriptive
/// message to help with debugging and error handling.
#[derive(Debug)]
pub enum ICloudError {
    /// Invalid URL format (not a valid iCloud shared album URL)
    InvalidUrl(String),

    /// Missing or invalid token in the URL
    InvalidToken(String),

    /// Network or API error when fetching album data
    FetchError(String),

    /// Error when parsing or processing photo data
    PhotoProcessingError(String),

    /// No derivatives found for a photo
    NoDerivativesError(String),

    /// Wraps an underlying error with context
    Context {
        /// The context message explaining what operation was being performed
        context: String,
        /// The underlying error that caused the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl ICloudError {
    /// Create a new error with additional context around an underlying error
    pub fn with_context<E, C>(error: E, context: C) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
        C: Into<String>,
    {
        Self::Context {
            context: context.into(),
            source: Box::new(error),
        }
    }

    /// Wrap a Result error with additional context
    pub fn context<T, E, C>(result: Result<T, E>, context: C) -> Result<T, Self>
    where
        E: std::error::Error + Send + Sync + 'static,
        C: Into<String>,
    {
        result.map_err(|err| Self::with_context(err, context))
    }
}

impl fmt::Display for ICloudError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl(msg) => write!(f, "Invalid iCloud URL: {}", msg),
            Self::InvalidToken(msg) => write!(f, "Invalid iCloud token: {}", msg),
            Self::FetchError(msg) => write!(f, "Failed to fetch album: {}", msg),
            Self::PhotoProcessingError(msg) => write!(f, "Error processing photo: {}", msg),
            Self::NoDerivativesError(msg) => write!(f, "No suitable derivatives: {}", msg),
            Self::Context { context, source } => write!(f, "{}: {}", context, source),
        }
    }
}

/// Implement the standard error trait for proper error handling
impl std::error::Error for ICloudError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Context { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Helper function to extract information from a derivative
fn extract_derivative_info(
    _key: &str,
    derivative: &icloud_album_rs::models::Derivative,
) -> Option<(String, u32, u32)> {
    derivative.url.as_ref().map(|url| {
        let width = derivative.width.unwrap_or(0);
        let height = derivative.height.unwrap_or(0);
        (url.clone(), width, height)
    })
}

/// Find best available derivative for a photo
///
/// Strategy:
/// 1. First try to find the "original" derivative as it typically offers the highest quality
/// 2. If original is not available, select the derivative with the highest resolution (width × height)
///
/// This ensures we always get the highest quality image available, regardless of the derivative name.
/// Note: It's normal for iCloud to provide different derivative types for different photos.
///
/// Returns a tuple of (url, width, height) or an error if no suitable derivative is found
/// Result containing URL, width, height, and MIME type
type DerivativeInfo = (String, u32, u32, String);

fn find_best_derivative(
    photo: &icloud_album_rs::models::Image,
) -> Result<DerivativeInfo, ICloudError> {
    debug!("Finding best derivative for photo: {}", photo.photo_guid);
    trace!("Photo has {} derivatives", photo.derivatives.len());

    // Log available derivatives for debugging
    if log::log_enabled!(log::Level::Trace) {
        for (key, derivative) in &photo.derivatives {
            trace!(
                "Available derivative: key={}, has_url={}, width={:?}, height={:?}",
                key,
                derivative.url.is_some(),
                derivative.width,
                derivative.height
            );
        }
    }

    // First try to find the original derivative as it's typically the highest quality
    if let Some((key, derivative)) = photo
        .derivatives
        .iter()
        .find(|(key, derivative)| key.contains("original") && derivative.url.is_some())
    {
        debug!("Found original derivative: {}", key);
        if let Some((url, width, height)) = extract_derivative_info(key, derivative) {
            let mime_type = determine_mime_type(&url, key);
            return Ok((url, width, height, mime_type));
        }
    }

    // If original not found, select the derivative with highest resolution (width × height)
    let mut best_derivative: Option<(&String, &icloud_album_rs::models::Derivative, u64)> = None;

    for (key, derivative) in photo.derivatives.iter() {
        // Skip derivatives without a URL
        if derivative.url.is_none() {
            continue;
        }

        // Get width and height, defaulting to 0 if missing
        let width = derivative.width.unwrap_or(0);
        let height = derivative.height.unwrap_or(0);

        // Calculate resolution (width × height)
        let resolution = width as u64 * height as u64;

        // Update best_derivative if this one has higher resolution
        match best_derivative {
            None => best_derivative = Some((key, derivative, resolution)),
            Some((_, _, best_res)) if resolution > best_res => {
                best_derivative = Some((key, derivative, resolution));
            }
            _ => {}
        }
    }

    // Use the highest resolution derivative if found
    if let Some((key, derivative, resolution)) = best_derivative {
        let result = extract_derivative_info(key, derivative);
        if let Some((url, width, height)) = result {
            let mime_type = determine_mime_type(&url, key);
            info!(
                "Selected highest resolution derivative: {} ({}×{} = {} pixels, MIME: {}) for photo {}",
                key, width, height, resolution, mime_type, photo.photo_guid
            );
            return Ok((url, width, height, mime_type));
        }
    }

    // If we reach here, no suitable derivative was found
    error!(
        "No derivatives with URL found for photo {}",
        photo.photo_guid
    );
    Err(ICloudError::NoDerivativesError(format!(
        "No derivatives with URL found for photo {}",
        photo.photo_guid
    )))
}

/// Fetches photos from an iCloud shared album using the icloud-album-rs crate
pub async fn fetch_album(album_url: &str) -> Result<Album> {
    info!("Fetching iCloud shared album: {}", album_url);

    // Define test token indicators explicitly
    const TEST_TOKEN_INDICATORS: [&str; 3] = ["#test", "#custom", "#example"];

    // Special handling for test or custom URLs
    let is_test_url = TEST_TOKEN_INDICATORS
        .iter()
        .any(|indicator| album_url.contains(indicator));
    if is_test_url {
        debug!("Detected test URL, using mock album data");
        return crate::mock::create_mock_album();
    }

    // Check if the URL seems valid before processing
    if !album_url.contains("icloud.com/sharedalbum")
        && !album_url.contains("share.icloud.com/photos")
    {
        error!("Invalid iCloud URL: {}", album_url);
        return Err(anyhow::anyhow!(ICloudError::InvalidUrl(
            "URL doesn't appear to be an iCloud shared album".to_string()
        )));
    }

    // Determine URL format
    let format = determine_url_format(album_url);
    debug!("URL format determined: {:?}", format);

    // Extract the token from the album URL
    debug!("Extracting token from URL");
    let token = match extract_token(album_url) {
        Ok(token) => {
            debug!(
                "Successfully extracted token: {}...",
                token.chars().take(8).collect::<String>()
            );
            token
        }
        Err(e) => {
            error!("Failed to extract token: {}", e);
            return Err(anyhow::anyhow!(ICloudError::InvalidToken(e.to_string())));
        }
    };

    // Fetch the album data using the icloud-album-rs crate
    info!("Fetching album data with token");
    let album_data = match icloud_album_rs::get_icloud_photos(&token).await {
        Ok(data) => {
            debug!("Successfully fetched album data");
            data
        }
        Err(e) => {
            error!("Failed to fetch iCloud album: {}", e);
            return Err(anyhow::anyhow!(ICloudError::FetchError(e.to_string())));
        }
    };

    // Create our Album struct from the icloud-album-rs response
    // If stream_name is empty, use a generic name with the token as a fallback
    let album_name = if album_data.metadata.stream_name.trim().is_empty() {
        let name = format!("iCloud Album {}", token.chars().take(8).collect::<String>());
        warn!("Album has no name, using generated name: {}", name);
        name
    } else {
        debug!("Using album name: {}", album_data.metadata.stream_name);
        album_data.metadata.stream_name.clone()
    };

    let mut album = Album::new(album_name);

    info!("Processing {} photos from album", album_data.photos.len());

    // Convert each photo from the icloud-album-rs format to our format
    let mut success_count = 0;
    let mut error_count = 0;
    let photo_count = album_data.photos.len();

    for (i, photo) in album_data.photos.into_iter().enumerate() {
        let photo_guid = photo.photo_guid.clone();
        trace!("Processing photo {}/{}: {}", i + 1, photo_count, photo_guid);

        let result = process_photo(&mut album, photo);

        match result {
            Ok(()) => {
                success_count += 1;
                trace!("Successfully processed photo: {}", photo_guid);
            }
            Err(e) => {
                error_count += 1;
                warn!("Failed to process photo: {}", e);
                // Continue with the next photo instead of failing the entire operation
                continue;
            }
        }
    }

    info!(
        "Processed {} photos: {} successful, {} errors",
        success_count + error_count,
        success_count,
        error_count
    );

    // If we didn't find any photos, that's suspicious
    if album.photos.is_empty() && error_count > 0 {
        error!(
            "Failed to process any photos from the album despite finding {} photos",
            error_count
        );
        return Err(anyhow::anyhow!(ICloudError::PhotoProcessingError(
            "Failed to process any photos from the album".to_string()
        )));
    }

    debug!("Returning album with {} photos", album.photos.len());
    Ok(album)
}

/// Parse a date string in RFC3339 format to a UTC DateTime
fn parse_photo_date(date_str: &str) -> Result<DateTime<Utc>, ICloudError> {
    debug!("Parsing date: {}", date_str);

    DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            warn!("Failed to parse date '{}': {}", date_str, e);
            ICloudError::PhotoProcessingError(format!("Failed to parse date {}: {}", date_str, e))
        })
}

/// Generate a checksum for a photo based on its GUID and URL
fn generate_photo_checksum(guid: &str, url: &str) -> String {
    let checksum = format!("{:x}", md5::compute(format!("{}:{}", guid, url)));
    trace!("Generated checksum: {}", checksum);
    checksum
}

/// Determine the MIME type of a photo from its URL and derivative key
fn determine_mime_type(url: &str, derivative_key: &str) -> String {
    if url.ends_with(".jpg") || url.ends_with(".jpeg") || derivative_key.contains("jpeg") {
        return "image/jpeg".to_string();
    } else if url.ends_with(".png") || derivative_key.contains("png") {
        return "image/png".to_string();
    } else if url.ends_with(".heic") || derivative_key.contains("heic") {
        return "image/heic".to_string();
    } else if url.ends_with(".gif") || derivative_key.contains("gif") {
        return "image/gif".to_string();
    } else if url.ends_with(".webp") || derivative_key.contains("webp") {
        return "image/webp".to_string();
    } else if url.ends_with(".mov") || derivative_key.contains("mov") || url.ends_with(".mp4") || derivative_key.contains("mp4") {
        return "video/mp4".to_string();
    }
    
    // Default to JPEG if unknown
    "image/jpeg".to_string()
}

/// Process a single photo from the iCloud API response and add it to the album
fn process_photo(
    album: &mut Album,
    photo: icloud_album_rs::models::Image,
) -> Result<(), ICloudError> {
    debug!("Processing photo: {}", photo.photo_guid);
    trace!(
        "Photo data: caption={:?}, derivatives_count={}",
        photo.caption,
        photo.derivatives.len()
    );

    // Find the best derivative with URL, width, height, and MIME type
    let (url, width, height, mime_type) = find_best_derivative(&photo)?;
    debug!("Found best derivative: width={}, height={}, mime_type={}", width, height, mime_type);

    // Parse the created date or use current time as fallback
    let created_at = match &photo.date_created {
        Some(date_str) => parse_photo_date(date_str)?,
        None => {
            let now = Utc::now();
            debug!("No date found, using current time: {}", now);
            now
        }
    };

    // Create a checksum and build the photo object
    let guid = photo.photo_guid.clone();
    let checksum = generate_photo_checksum(&guid, &url);
    
    // Determine the correct file extension based on MIME type
    let extension = match mime_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/heic" => "heic",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "video/mp4" => "mp4",
        _ => "jpg", // Default to jpg for unknown types
    };

    let icloud_photo = Photo {
        guid: photo.photo_guid,
        filename: format!("{}.{}", guid, extension),
        caption: photo.caption.clone(),
        created_at,
        checksum,
        url,
        width,
        height,
        mime_type,
    };

    // Add the photo to our album
    debug!("Adding photo to album: {}", icloud_photo.guid);
    album.photos.insert(icloud_photo.guid.clone(), icloud_photo);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_album_new() {
        let album = Album::new("Test Album".to_string());
        assert_eq!(album.name, "Test Album");
        assert!(album.photos.is_empty());
    }

    #[test]
    fn test_photo_serialization() -> Result<()> {
        let photo = Photo {
            guid: "test_guid".to_string(),
            filename: "test.jpg".to_string(),
            caption: Some("Test Caption".to_string()),
            created_at: "2023-01-01T12:00:00Z".parse::<DateTime<Utc>>()?,
            checksum: "abcdef1234567890".to_string(),
            url: "https://example.com/test.jpg".to_string(),
            width: 1920,
            height: 1080,
            mime_type: "image/jpeg".to_string(),
        };

        let serialized = serde_json::to_string(&photo)?;
        let deserialized: Photo = serde_json::from_str(&serialized)?;

        assert_eq!(deserialized.guid, "test_guid");
        assert_eq!(deserialized.filename, "test.jpg");
        assert_eq!(deserialized.caption, Some("Test Caption".to_string()));
        assert_eq!(deserialized.checksum, "abcdef1234567890");

        Ok(())
    }

    #[test]
    fn test_album_serialization() -> Result<()> {
        let mut album = Album::new("Test Album".to_string());
        let photo = Photo {
            guid: "test_guid".to_string(),
            filename: "test.jpg".to_string(),
            caption: Some("Test Caption".to_string()),
            created_at: "2023-01-01T12:00:00Z".parse::<DateTime<Utc>>()?,
            checksum: "abcdef1234567890".to_string(),
            url: "https://example.com/test.jpg".to_string(),
            width: 1920,
            height: 1080,
            mime_type: "image/jpeg".to_string(),
        };

        album.photos.insert(photo.guid.clone(), photo);

        let serialized = serde_json::to_string(&album)?;
        let deserialized: Album = serde_json::from_str(&serialized)?;

        assert_eq!(deserialized.name, "Test Album");
        assert_eq!(deserialized.photos.len(), 1);
        assert!(deserialized.photos.contains_key("test_guid"));

        Ok(())
    }
}
