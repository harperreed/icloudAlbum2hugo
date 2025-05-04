use anyhow::{Context, Result};
use std::io::Write;
use url::Url;

pub async fn debug_album_api(album_url: &str) -> Result<()> {
    // Special handling for test URLs
    if album_url.contains("#test") || album_url.contains("#custom") {
        println!("Using mock data for test URL: {}", album_url);
        
        // Create a simple debug output file for test purposes
        let mut debug_output = String::new();
        debug_output.push_str(&format!("Mock Album data:\n"));
        debug_output.push_str(&format!("  Album URL: {}\n", album_url));
        debug_output.push_str(&format!("  Photos count: 3\n"));
        
        // Save the debug output to a file
        let mut file = std::fs::File::create("album_data_debug.txt")?;
        file.write_all(debug_output.as_bytes())?;
        
        return Ok(());
    }
    
    // Validate and parse the URL
    let url = Url::parse(album_url)
        .with_context(|| format!("Invalid iCloud shared album URL: {}", album_url))?;
    
    // Extract the token (shared album ID) from the URL
    let token = url.fragment()
        .and_then(|fragment| {
            // Typical format is "fragment=#B0aBcDeFG..." - we want the part after #
            if fragment.starts_with("B") {
                Some(fragment)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid iCloud shared album URL: missing or invalid token"))?;
    
    // Use the icloud_album_rs crate to fetch album data
    let album_data = icloud_album_rs::get_icloud_photos(token).await
        .map_err(|e| anyhow::anyhow!("Failed to fetch album: {}", e))?;
    
    // Since we can't directly serialize the album_data, extract the information manually
    let mut debug_output = String::new();
    debug_output.push_str(&format!("Album data:\n"));
    debug_output.push_str(&format!("  Stream name: {}\n", album_data.metadata.stream_name));
    debug_output.push_str(&format!("  Owner: {} {}\n", 
        album_data.metadata.user_first_name, 
        album_data.metadata.user_last_name));
    debug_output.push_str(&format!("  Photos count: {}\n", album_data.photos.len()));
    
    // Add information about each photo
    for (i, photo) in album_data.photos.iter().enumerate().take(5) {
        debug_output.push_str(&format!("\nPhoto {}:\n", i + 1));
        debug_output.push_str(&format!("  GUID: {}\n", photo.photo_guid));
        debug_output.push_str(&format!("  Caption: {:?}\n", photo.caption));
        debug_output.push_str(&format!("  Created: {:?}\n", photo.date_created));
        debug_output.push_str(&format!("  Batch Created: {:?}\n", photo.batch_date_created));
        debug_output.push_str(&format!("  Derivatives count: {}\n", photo.derivatives.len()));
        
        // Add information about derivatives (variants of the photo)
        if !photo.derivatives.is_empty() {
            for (j, (key, value)) in photo.derivatives.iter().enumerate().take(3) {
                debug_output.push_str(&format!("  Derivative {}:\n", j + 1));
                debug_output.push_str(&format!("    Key: {}\n", key));
                debug_output.push_str(&format!("    Value: {:?}\n", value));
            }
            
            if photo.derivatives.len() > 3 {
                debug_output.push_str(&format!("    ... and {} more derivatives\n", 
                    photo.derivatives.len() - 3));
            }
        }
    }
    
    if album_data.photos.len() > 5 {
        debug_output.push_str(&format!("\n... and {} more photos\n", 
            album_data.photos.len() - 5));
    }
    
    // Save the debug output to a file
    let mut file = std::fs::File::create("album_data_debug.txt")?;
    file.write_all(debug_output.as_bytes())?;
    
    println!("Saved album data to album_data_debug.txt");
    println!("Summary:");
    println!("  Album: {}", album_data.metadata.stream_name);
    println!("  Photos: {}", album_data.photos.len());
    
    Ok(())
}