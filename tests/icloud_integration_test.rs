//! Integration tests for real iCloud API
//!
//! These tests are optional and only run when the ICLOUD_TEST_TOKEN environment
//! variable is set. They test the real iCloud API integration using the provided token.
//!
//! Run with:
//! ICLOUD_TEST_TOKEN=B2T5VaUrzMLxwU cargo test --test icloud_integration_test -- --nocapture

use icloudAlbum2hugo::icloud;
use log::{info, warn};
use std::env;
use std::sync::Once;

/// Token for testing - can be overridden with ICLOUD_TEST_TOKEN environment variable
const DEFAULT_TEST_TOKEN: &str = "B2T5VaUrzMLxwU";

// Initialize the logger only once
static INIT: Once = Once::new();

/// Initialize the logger for tests
fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Seconds))
            .is_test(true)
            .init();
    });
}

/// Get the iCloud token from environment or use default
fn get_test_token() -> String {
    env::var("ICLOUD_TEST_TOKEN").unwrap_or_else(|_| DEFAULT_TEST_TOKEN.to_string())
}

/// Create a test URL from a token
fn create_test_url(token: &str) -> String {
    format!("https://www.icloud.com/sharedalbum/#{token}")
}

#[tokio::test]
async fn test_real_icloud_fetch() {
    // Initialize the logger
    init_logger();

    // Get the test token
    let token = get_test_token();

    // Skip test if running in CI or explicitly disabled
    if env::var("CI").is_ok() || env::var("SKIP_ICLOUD_TEST").is_ok() {
        info!("Skipping iCloud integration test in CI environment");
        return;
    }

    if token == DEFAULT_TEST_TOKEN {
        info!("Using default test token - override with ICLOUD_TEST_TOKEN env var if needed");
    } else {
        info!("Using custom test token from environment");
    }

    // Create the URL
    let url = create_test_url(&token);
    info!("Testing with URL: {url}");

    // Fetch the album
    let result = icloud::fetch_album(&url).await;

    // Check the result
    match result {
        Ok(album) => {
            info!("✅ Successfully fetched album: '{}'", album.name);
            info!("Found {} photos", album.photos.len());

            if album.photos.is_empty() {
                warn!("⚠️ Album has no photos!");
            } else {
                info!("Album details:");
                info!("  • Album name: {}", album.name);
                info!("  • Photo count: {}", album.photos.len());

                // Print details of the first 3 photos
                for (i, (guid, photo)) in album.photos.iter().take(3).enumerate() {
                    info!("  • Photo {}: {}", i + 1, guid);
                    info!("    - Caption: {:?}", photo.caption);
                    info!("    - Created: {}", photo.created_at);
                    info!("    - Dimensions: {}x{}", photo.width, photo.height);
                }

                // If there are more than 3 photos, indicate this
                if album.photos.len() > 3 {
                    info!("    ... and {} more photos", album.photos.len() - 3);
                }

                info!("✅ Test passed: Successfully fetched and processed album");
            }
        }
        Err(err) => {
            panic!("❌ Failed to fetch iCloud album: {err}");
        }
    }
}

/// Test handling of invalid iCloud URLs
#[tokio::test]
async fn test_invalid_urls() {
    // Initialize the logger
    init_logger();

    // Skip test if running in CI or explicitly disabled
    if env::var("CI").is_ok() || env::var("SKIP_ICLOUD_TEST").is_ok() {
        info!("Skipping iCloud integration test in CI environment");
        return;
    }

    let test_cases = [
        "https://www.example.com",                    // Not an iCloud URL
        "https://icloud.com/sharedalbum/not-a-token", // Missing proper token format
        "https://www.icloud.com/sharedalbum/#",       // Missing token
        "B2T5VaUrzMLxwU",
    ];

    let mut passed = 0;
    let total = test_cases.len();

    for (i, url) in test_cases.iter().enumerate() {
        info!("Testing invalid URL case {}/{}: {}", i + 1, total, url);

        let result = icloud::fetch_album(url).await;

        match result {
            Ok(_) => {
                panic!("❌ Expected error for invalid URL '{url}', but got success");
            }
            Err(err) => {
                info!("✅ Correctly handled invalid URL: {err}");
                passed += 1;
            }
        }
    }

    info!("✅ All {passed} invalid URL tests passed");
}
