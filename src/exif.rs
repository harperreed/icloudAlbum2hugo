//! EXIF metadata extraction for icloud2hugo.
//!
//! This module handles extracting and processing EXIF metadata from photos.
//! It provides functionality to extract camera information, date/time, GPS coordinates,
//! and other technical data from image files.
//!
//! The core function `extract_exif` processes a JPEG image and returns an `ExifMetadata`
//! struct containing all the extracted information. This module also includes helper
//! functions for parsing specific EXIF tags and fuzzing GPS coordinates for privacy.

use anyhow::{Context, Result};
use exif::{In, Tag, Value, Exif};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use chrono::{DateTime, TimeZone, Utc};
use rand::Rng;
use log::warn;

/// Represents the extracted EXIF metadata from a photo
#[derive(Debug, Clone, Default)]
pub struct ExifMetadata {
    /// Make of the camera used to take the photo (e.g., "Apple")
    pub camera_make: Option<String>,
    /// Model of the camera used (e.g., "iPhone 15 Pro")
    pub camera_model: Option<String>,
    /// When the photo was taken (from EXIF data)
    pub date_time: Option<DateTime<Utc>>,
    /// Original latitude from EXIF data
    pub latitude: Option<f64>,
    /// Original longitude from EXIF data
    pub longitude: Option<f64>,
    /// Fuzzed latitude (slightly offset from original for privacy)
    pub fuzzed_latitude: Option<f64>,
    /// Fuzzed longitude (slightly offset from original for privacy)
    pub fuzzed_longitude: Option<f64>,
    /// ISO speed rating used for the photo
    pub iso: Option<u32>,
    /// Exposure time in seconds (e.g., 1/120)
    pub exposure_time: Option<String>,
    /// F-number or aperture value (e.g., f/2.8)
    pub f_number: Option<f32>,
    /// Focal length in millimeters (e.g., 4.2mm)
    pub focal_length: Option<f32>,
}

/// Extracts EXIF metadata from a JPEG image file
pub fn extract_exif(image_path: &Path) -> Result<ExifMetadata> {
    // Default metadata in case we can't read EXIF data
    let mut metadata = ExifMetadata::default();
    
    // Open the file
    let file = File::open(image_path)
        .with_context(|| format!("Failed to open image file at {}", image_path.display()))?;
    
    let mut bufreader = BufReader::new(&file);
    
    // Try to extract EXIF data
    let exif = match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(exif) => exif,
        Err(e) => {
            // Log the error but return default metadata
            warn!("Could not extract EXIF data from {}: {}", image_path.display(), e);
            return Ok(metadata);
        }
    };
    
    // Extract basic camera information
    metadata.camera_make = get_exif_string(&exif, Tag::Make);
    metadata.camera_model = get_exif_string(&exif, Tag::Model);
    
    // Extract date/time
    if let Some(date_str) = get_exif_string(&exif, Tag::DateTimeOriginal) {
        metadata.date_time = parse_exif_datetime(&date_str);
    }
    
    // Extract GPS coordinates
    extract_gps_coordinates(&exif, &mut metadata);
    
    // Apply fuzzed coordinates if GPS data exists
    if metadata.latitude.is_some() && metadata.longitude.is_some() {
        fuzz_coordinates(&mut metadata);
    }
    
    // Extract other photo information
    metadata.iso = get_exif_u32(&exif, Tag::ISOSpeed);
    metadata.exposure_time = get_exif_rational_as_string(&exif, Tag::ExposureTime);
    metadata.f_number = get_exif_f32(&exif, Tag::FNumber);
    metadata.focal_length = get_exif_f32(&exif, Tag::FocalLength);
    
    Ok(metadata)
}

/// Helper function to extract a string from EXIF data
fn get_exif_string(exif: &Exif, tag: Tag) -> Option<String> {
    if let Some(field) = exif.get_field(tag, In::PRIMARY) {
        if let Value::Ascii(ref vec) = field.value {
            if let Some(string) = vec.first() {
                return Some(String::from_utf8_lossy(string).to_string());
            }
        }
    }
    None
}

/// Helper function to extract a u32 value from EXIF data
fn get_exif_u32(exif: &Exif, tag: Tag) -> Option<u32> {
    if let Some(field) = exif.get_field(tag, In::PRIMARY) {
        match &field.value {
            Value::Short(vec) => vec.first().map(|&v| u32::from(v)),
            Value::Long(vec) => vec.first().copied(),
            _ => None,
        }
    } else {
        None
    }
}

/// Helper function to extract a f32 value from EXIF data
fn get_exif_f32(exif: &Exif, tag: Tag) -> Option<f32> {
    if let Some(field) = exif.get_field(tag, In::PRIMARY) {
        if let Value::Rational(ref vec) = field.value {
            if let Some(rational) = vec.first() {
                return Some(rational.to_f32());
            }
        }
    }
    None
}

/// Helper function to extract a rational value and format it as a string
fn get_exif_rational_as_string(exif: &Exif, tag: Tag) -> Option<String> {
    if let Some(field) = exif.get_field(tag, In::PRIMARY) {
        if let Value::Rational(ref vec) = field.value {
            if let Some(rational) = vec.first() {
                if rational.denom == 1 {
                    // When denominator is 1, just show the numerator (e.g., "30" seconds)
                    return Some(format!("{}", rational.num));
                } else if rational.num == 0 {
                    // Handle the case where numerator is 0
                    return Some("0".to_string());
                } else if rational.denom % rational.num == 0 {
                    // When denominator is a multiple of numerator (e.g., 1/60, 1/125)
                    return Some(format!("1/{}", rational.denom / rational.num));
                } else {
                    // General case: display as num/denom fraction
                    return Some(format!("{}/{}", rational.num, rational.denom));
                }
            }
        }
    }
    None
}

/// Parse EXIF DateTime format (e.g., "2023:12:25 15:30:00") into a UTC DateTime
fn parse_exif_datetime(date_str: &str) -> Option<DateTime<Utc>> {
    // Standard EXIF date format is "YYYY:MM:DD HH:MM:SS"
    
    // First, split by space to separate date and time parts
    let date_time_parts: Vec<&str> = date_str.split(' ').collect();
    if date_time_parts.len() != 2 {
        return None; // Invalid format
    }
    
    let date_part = date_time_parts[0];
    let time_part = date_time_parts[1];
    
    // Split date part by colon
    let date_components: Vec<&str> = date_part.split(':').collect();
    if date_components.len() != 3 {
        return None; // Invalid date format
    }
    
    // Split time part by colon
    let time_components: Vec<&str> = time_part.split(':').collect();
    if time_components.len() != 3 {
        return None; // Invalid time format
    }
    
    // Parse components
    let year = date_components[0].parse::<i32>().ok()?;
    let month = date_components[1].parse::<u32>().ok()?;
    let day = date_components[2].parse::<u32>().ok()?;
    let hour = time_components[0].parse::<u32>().ok()?;
    let minute = time_components[1].parse::<u32>().ok()?;
    let second = time_components[2].parse::<u32>().ok()?;
    
    // Create the DateTime object
    Utc.with_ymd_and_hms(year, month, day, hour, minute, second).single()
}

/// Extract GPS coordinates from EXIF data
fn extract_gps_coordinates(exif: &Exif, metadata: &mut ExifMetadata) {
    // Function to convert GPS coordinates from DMS (degrees, minutes, seconds) to decimal degrees
    let dms_to_decimal = |degrees: f64, minutes: f64, seconds: f64, direction: &str| -> f64 {
        let mut decimal = degrees + minutes/60.0 + seconds/3600.0;
        if direction == "S" || direction == "W" {
            decimal = -decimal;
        }
        decimal
    };
    
    // Extract latitude
    let mut lat_deg = 0.0;
    let mut lat_min = 0.0;
    let mut lat_sec = 0.0;
    let mut lat_dir = String::new();
    
    if let Some(field) = exif.get_field(Tag::GPSLatitude, In::PRIMARY) {
        if let Value::Rational(ref vec) = field.value {
            if vec.len() >= 3 {
                lat_deg = vec[0].to_f64();
                lat_min = vec[1].to_f64();
                lat_sec = vec[2].to_f64();
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY) {
        if let Value::Ascii(ref vec) = field.value {
            if let Some(dir) = vec.first() {
                lat_dir = String::from_utf8_lossy(dir).to_string();
            }
        }
    }
    
    // Extract longitude
    let mut lon_deg = 0.0;
    let mut lon_min = 0.0;
    let mut lon_sec = 0.0;
    let mut lon_dir = String::new();
    
    if let Some(field) = exif.get_field(Tag::GPSLongitude, In::PRIMARY) {
        if let Value::Rational(ref vec) = field.value {
            if vec.len() >= 3 {
                lon_deg = vec[0].to_f64();
                lon_min = vec[1].to_f64();
                lon_sec = vec[2].to_f64();
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY) {
        if let Value::Ascii(ref vec) = field.value {
            if let Some(dir) = vec.first() {
                lon_dir = String::from_utf8_lossy(dir).to_string();
            }
        }
    }
    
    // Convert to decimal degrees
    if !lat_dir.is_empty() && !lon_dir.is_empty() {
        let latitude = dms_to_decimal(lat_deg, lat_min, lat_sec, &lat_dir);
        let longitude = dms_to_decimal(lon_deg, lon_min, lon_sec, &lon_dir);
        
        metadata.latitude = Some(latitude);
        metadata.longitude = Some(longitude);
    }
}

/// Apply a small random offset to coordinates for privacy
fn fuzz_coordinates(metadata: &mut ExifMetadata) {
    let mut rng = rand::thread_rng();
    
    // Apply a random offset of up to ±0.001 degrees (about ±111 meters for latitude)
    if let Some(lat) = metadata.latitude {
        let offset = rng.gen_range(-0.001..0.001);
        metadata.fuzzed_latitude = Some(lat + offset);
    }
    
    if let Some(lon) = metadata.longitude {
        let offset = rng.gen_range(-0.001..0.001);
        metadata.fuzzed_longitude = Some(lon + offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use chrono::{Datelike, Timelike};
    
    /// Creates a small test JPEG with minimal EXIF data for testing
    fn create_test_jpeg_with_exif(path: &Path) -> Result<()> {
        // This is a minimal JPEG file with basic EXIF data
        // In a real implementation, we would include a real sample image file
        // Here we're just creating a placeholder file
        let test_data = b"JFIF\0EXIF\0Test JPEG with EXIF data";
        let mut file = File::create(path)?;
        file.write_all(test_data)?;
        Ok(())
    }
    
    #[test]
    fn test_extract_exif_with_missing_file() {
        let result = extract_exif(Path::new("/nonexistent/path.jpg"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_exif_with_minimal_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let image_path = temp_dir.path().join("test.jpg");
        
        create_test_jpeg_with_exif(&image_path)?;
        
        let metadata = extract_exif(&image_path)?;
        
        // We expect default values since our test file doesn't have real EXIF data
        assert!(metadata.camera_make.is_none());
        assert!(metadata.camera_model.is_none());
        assert!(metadata.date_time.is_none());
        assert!(metadata.latitude.is_none());
        assert!(metadata.longitude.is_none());
        
        Ok(())
    }
    
    #[test]
    fn test_fuzz_coordinates() {
        let mut metadata = ExifMetadata {
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            ..Default::default()
        };
        
        fuzz_coordinates(&mut metadata);
        
        // Check that fuzzed coordinates were set and are different
        assert!(metadata.fuzzed_latitude.is_some());
        assert!(metadata.fuzzed_longitude.is_some());
        
        // Check that the fuzzed value is within expected range
        if let Some(orig_lat) = metadata.latitude {
            if let Some(fuzz_lat) = metadata.fuzzed_latitude {
                assert!((orig_lat - fuzz_lat).abs() <= 0.001);
                assert!((orig_lat - fuzz_lat).abs() > 0.0);
            }
        }
        
        if let Some(orig_lon) = metadata.longitude {
            if let Some(fuzz_lon) = metadata.fuzzed_longitude {
                assert!((orig_lon - fuzz_lon).abs() <= 0.001);
                assert!((orig_lon - fuzz_lon).abs() > 0.0);
            }
        }
    }
    
    #[test]
    fn test_parse_exif_datetime() {
        // Test valid format
        let date_str = "2023:12:25 15:30:00";
        let result = parse_exif_datetime(date_str);
        assert!(result.is_some());
        
        if let Some(dt) = result {
            assert_eq!(dt.year(), 2023);
            assert_eq!(dt.month(), 12);
            assert_eq!(dt.day(), 25);
            assert_eq!(dt.hour(), 15);
            assert_eq!(dt.minute(), 30);
            assert_eq!(dt.second(), 0);
        }
        
        // Test invalid format
        let invalid = "2023-12-25";
        let result = parse_exif_datetime(invalid);
        assert!(result.is_none());
    }
}