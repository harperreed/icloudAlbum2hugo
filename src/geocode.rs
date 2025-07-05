//! Reverse geocoding for icloud2hugo.
//!
//! This module provides functionality to convert geographic coordinates (latitude/longitude)
//! into human-readable location information (city, state, country).
//!
//! It defines the `Location` struct to store formatted location data and the
//! `GeocodingService` trait as an interface for different geocoding implementations.
//! The current implementation uses a mock service that returns predefined locations
//! for certain coordinate ranges, but this could be extended to use real geocoding APIs.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a geographic location with address components
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Location {
    /// The full formatted address (e.g., "Chicago, IL, USA")
    pub formatted_address: String,
    /// City or locality name
    pub city: Option<String>,
    /// State, province, or administrative area
    pub state: Option<String>,
    /// Country name
    pub country: Option<String>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formatted_address)
    }
}

/// Interface for reverse geocoding services
pub trait GeocodingService {
    /// Convert latitude and longitude to a location
    fn reverse_geocode(&self, latitude: f64, longitude: f64) -> Result<Location>;
}

/// Mock geocoding service for testing and offline use
pub struct MockGeocodingService;

impl GeocodingService for MockGeocodingService {
    fn reverse_geocode(&self, latitude: f64, longitude: f64) -> Result<Location> {
        // For mocking purposes, we'll create some predefined locations based on coordinate ranges

        // Chicago area (roughly)
        if latitude > 41.5 && latitude < 42.0 && longitude > -88.0 && longitude < -87.5 {
            return Ok(Location {
                formatted_address: "Chicago, IL, USA".to_string(),
                city: Some("Chicago".to_string()),
                state: Some("Illinois".to_string()),
                country: Some("United States".to_string()),
            });
        }

        // New York area (roughly)
        if latitude > 40.5 && latitude < 41.0 && longitude > -74.5 && longitude < -73.5 {
            return Ok(Location {
                formatted_address: "New York, NY, USA".to_string(),
                city: Some("New York".to_string()),
                state: Some("New York".to_string()),
                country: Some("United States".to_string()),
            });
        }

        // San Francisco area (roughly)
        if latitude > 37.5 && latitude < 38.0 && longitude > -123.0 && longitude < -122.0 {
            return Ok(Location {
                formatted_address: "San Francisco, CA, USA".to_string(),
                city: Some("San Francisco".to_string()),
                state: Some("California".to_string()),
                country: Some("United States".to_string()),
            });
        }

        // London area (roughly)
        if latitude > 51.0 && latitude < 52.0 && longitude > -0.5 && longitude < 0.5 {
            return Ok(Location {
                formatted_address: "London, England, UK".to_string(),
                city: Some("London".to_string()),
                state: Some("England".to_string()),
                country: Some("United Kingdom".to_string()),
            });
        }

        // For any other coordinates, return a generic location based on the quadrant
        let ns = if latitude >= 0.0 { "North" } else { "South" };
        let ew = if longitude >= 0.0 { "East" } else { "West" };

        Ok(Location {
            formatted_address: format!("{ns} {ew} at {latitude:.4}, {longitude:.4}"),
            city: None,
            state: None,
            country: None,
        })
    }
}

/// Factory function to create a geocoding service
pub fn create_geocoding_service() -> Box<dyn GeocodingService> {
    // Currently we only support the mock service, but this could be extended
    // to use an actual geocoding API in the future
    Box::new(MockGeocodingService)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_geocoding_chicago() {
        let service = MockGeocodingService;
        let result = service.reverse_geocode(41.8781, -87.6298).unwrap();

        assert_eq!(result.formatted_address, "Chicago, IL, USA");
        assert_eq!(result.city, Some("Chicago".to_string()));
        assert_eq!(result.state, Some("Illinois".to_string()));
        assert_eq!(result.country, Some("United States".to_string()));
    }

    #[test]
    fn test_mock_geocoding_unknown_location() {
        let service = MockGeocodingService;
        let result = service.reverse_geocode(0.0, 0.0).unwrap();

        assert_eq!(result.formatted_address, "North East at 0.0000, 0.0000");
        assert_eq!(result.city, None);
        assert_eq!(result.state, None);
        assert_eq!(result.country, None);
    }

    #[test]
    fn test_location_display() {
        let location = Location {
            formatted_address: "Test City, Test State, Test Country".to_string(),
            city: Some("Test City".to_string()),
            state: Some("Test State".to_string()),
            country: Some("Test Country".to_string()),
        };

        assert_eq!(
            format!("{}", location),
            "Test City, Test State, Test Country"
        );
    }
}
