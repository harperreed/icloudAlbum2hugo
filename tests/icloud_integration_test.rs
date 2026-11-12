//! Integration tests for real iCloud API
//!
//! Tests that talk to the live iCloud API are optional and only run when the
//! `ICLOUD_TEST_TOKEN` environment variable is set. Without it they skip, so a
//! plain `cargo test` does not depend on network access or on the validity of a
//! hard-coded share token.
//!
//! Run with:
//! ICLOUD_TEST_TOKEN=<shared-album-token> cargo test --test icloud_integration_test -- --nocapture

use icloudAlbum2hugo::icloud;
use log::{info, warn};
use std::env;
use std::sync::Once;

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

/// Get the iCloud token from the environment, if one was provided
fn get_test_token() -> Option<String> {
    env::var("ICLOUD_TEST_TOKEN")
        .ok()
        .filter(|token| !token.trim().is_empty())
}

/// Whether tests that require live network access to iCloud should be skipped
fn skip_network_tests() -> bool {
    env::var("CI").is_ok() || env::var("SKIP_ICLOUD_TEST").is_ok()
}

/// Create a test URL from a token
fn create_test_url(token: &str) -> String {
    format!("https://www.icloud.com/sharedalbum/#{token}")
}

#[tokio::test]
async fn test_real_icloud_fetch() {
    // Initialize the logger
    init_logger();

    // Skip test if running in CI or explicitly disabled
    if skip_network_tests() {
        info!("Skipping iCloud integration test in CI environment");
        return;
    }

    // This test needs a real share token; without one there is nothing to fetch.
    let Some(token) = get_test_token() else {
        info!("Skipping iCloud integration test: set ICLOUD_TEST_TOKEN to run it");
        return;
    };

    info!("Using test token from environment");

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
    if skip_network_tests() {
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
