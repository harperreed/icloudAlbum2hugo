# icloudAlbum2hugo

A command-line tool that syncs photos from iCloud Shared Albums to a Hugo site.

This tool fetches photos from a shared iCloud album, extracts EXIF data, performs reverse geocoding (when location data is available), and organizes everything into Hugo page bundles under `content/photostream/<photo_id>/`.

![Banner Image - icloudAlbum2hugo](/assets/banner.png)

## Features

- ✨ Downloads new/updated photos at full resolution
- 🗑️ Removes photos that no longer exist in the album
- 📷 Extracts EXIF metadata (camera info, date/time, location)
- 🌎 Performs reverse geocoding with privacy-focused location fuzzing
- 📁 Creates Hugo page bundles with comprehensive frontmatter
- 📑 Maintains a master YAML index file for efficient syncing
- 🔄 Incremental updates - only downloads what's changed
- 📊 Provides detailed status reporting

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Detailed Usage](#detailed-usage)
  - [Command: init](#command-init)
  - [Command: sync](#command-sync)
  - [Command: status](#command-status)
- [Configuration Options](#configuration-options)
- [Hugo Integration](#hugo-integration)
  - [Frontmatter Fields](#frontmatter-fields)
  - [Title Formatting](#title-formatting)
  - [Hugo Theme Integration](#hugo-theme-integration)
- [Troubleshooting](#troubleshooting)
- [Advanced Usage](#advanced-usage)
- [Development](#development)
- [License](#license)

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.65 or newer
- An iCloud shared album URL

### Installing from Source

```bash
# Clone the repository
git clone https://github.com/harperreed/icloudAlbum2hugo.git
cd icloudAlbum2hugo

# Build with cargo
cargo build --release

# Move the binary to a location in your PATH (optional)
cp target/release/icloud2hugo ~/.local/bin/
```

### Installing via Cargo

```bash
cargo install icloudAlbum2hugo
```

## Quick Start

```bash
# 1. Initialize configuration
icloudAlbum2hugo init

# 2. Edit config.yaml and add your iCloud shared album URL
nano config.yaml

# 3. Sync photos from iCloud to your Hugo site
icloudAlbum2hugo sync

# 4. Check that everything is in sync
icloudAlbum2hugo status
```

## Detailed Usage

### Command: init

Creates a default configuration file in the current directory.

```bash
# Create default config.yaml
icloudAlbum2hugo init

# Create config at a custom location
icloudAlbum2hugo init --config ~/my-hugo-site/custom-config.yaml

# Overwrite existing config file
icloudAlbum2hugo init --force
```

### Command: sync

Synchronizes photos from your iCloud shared album to your Hugo site.

```bash
# Sync using default config
icloudAlbum2hugo sync

# Sync using a custom config file
icloudAlbum2hugo sync --config ~/my-hugo-site/custom-config.yaml
```

During synchronization, the following steps are performed:

1. Load configuration and existing photo index
2. Fetch the iCloud shared album data
3. Download new photos not in your local index
4. Update photos that have changed in the remote album
5. Remove photos no longer in the shared album
6. Extract EXIF data from each photo
7. Perform reverse geocoding for photos with GPS coordinates
8. Apply privacy fuzzing to location data
9. Create or update Hugo page bundles with frontmatter
10. Update the master index.yaml file

Typical output looks like:

```
┌─────────────────────────────────────────────┐
│        icloudAlbum2hugo Photo Sync         │
└─────────────────────────────────────────────┘

📋 Configuration:
  • Album URL: https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW
  • Output directory: content/photostream
  • Data file: data/photos/index.yaml

📂 Loading photo index from data/photos/index.yaml...
  • Photo index loaded with 42 photos

🔄 Fetching album data from iCloud...
  • Album 'My Vacation Photos' fetched with 45 photos

📷 Syncing photos to local filesystem...

💾 Saving photo index to data/photos/index.yaml...
  • Photo index saved successfully

✅ Sync completed successfully:
  • Added: 3
  • Updated: 0
  • Unchanged: 42
  • Deleted: 0
  • Total photos in index: 45
```

### Command: status

Shows the current status of your local photos compared to the remote album.

```bash
# Check status using default config
icloudAlbum2hugo status

# Check status using custom config
icloudAlbum2hugo status --config ~/my-hugo-site/custom-config.yaml
```

The status command provides a detailed report including:

- How many photos are in sync
- New photos available to download
- Photos that need updating
- Photos that will be removed
- Statistics about EXIF and location data

Typical output looks like:

```
┌─────────────────────────────────────────────┐
│            icloudAlbum2hugo Status          │
└─────────────────────────────────────────────┘

📋 Configuration:
  • Album URL: https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW
  • Output directory: content/photostream
  • Data file: data/photos/index.yaml

📂 Loading photo index from data/photos/index.yaml...
  • Photo index loaded with 42 photos
  • Last updated: 2023-07-15T10:24:35Z
  • Photos with EXIF data: 38/42
  • Photos with GPS coordinates: 32/42
  • Photos with location info: 29/42

🔄 Fetching album data from iCloud...
  • Album 'My Vacation Photos' fetched with 45 photos

📊 Status Summary:
  • Local photos: 42
  • Remote photos: 45
  • Photos in sync: 42
  • New photos to download: 3
  • Photos to update: 0
  • Photos to remove: 0

🆕 New photos to download:
  1. Pmc7WgZhHjkSW9Ew - Beach sunset
  2. QM732LSkhGkDfgT8 - Mountain view
  3. RtvBc7HjnmL9sDf4 - Family dinner

📋 Suggested Actions:
  • Run 'icloudAlbum2hugo sync' to update your local files
```

## Configuration Options

The configuration file (`config.yaml`) supports the following options:

```yaml
# Required settings
album_url: "https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW"  # Your iCloud shared album URL
out_dir: "content/photostream"                    # Output directory for Hugo page bundles
data_file: "data/photos/index.yaml"               # Path to the photo index file

# Optional settings
fuzz_meters: 100.0                                # Distance in meters to fuzz location (default: 100.0)
```

### Finding Your iCloud Shared Album URL

1. In the iCloud Photos app or iCloud.com, navigate to the shared album
2. Click on the "Share" button
3. Select "Copy Link"
4. Paste this URL into your config.yaml file

The URL should look like: `https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW`

## Hugo Integration

### Directory Structure

The tool creates a clean Hugo site structure that works with most themes:

```
your-hugo-site/
├── config.yaml            # Your Hugo config
├── content/
│   └── photostream/       # Photo content directory
│       ├── photo123456/   # Page bundle for one photo
│       │   ├── index.md   # Frontmatter + caption
│       │   └── original.jpg
│       └── photo789012/   # Page bundle for another photo
│           ├── index.md
│           └── original.jpg
└── data/
    └── photos/
        └── index.yaml     # Master index of all photos
```

### Frontmatter Fields

Each `index.md` file contains comprehensive frontmatter:

```yaml
---
title: "Photo taken on July 15, 2023"  # Caption or auto-generated title
date: 2023-07-15T14:30:22+0000         # Photo creation date
guid: "photo123456"                     # Unique ID from iCloud
original_filename: "IMG_1234.jpg"       # Original filename
width: 4032                             # Image width in pixels
height: 3024                            # Image height in pixels

# EXIF data (if available)
camera_make: "Apple"                    # Camera manufacturer
camera_model: "iPhone 12 Pro"           # Camera model
exif_date: 2023-07-15T14:30:22+0000     # Date from EXIF data

# Location data (if available and with privacy fuzzing)
original_latitude: 41.878765            # Original GPS latitude
original_longitude: -87.635987          # Original GPS longitude
latitude: 41.878901                     # Fuzzed latitude for privacy
longitude: -87.636123                   # Fuzzed longitude for privacy
location: "Chicago, IL, USA"            # Formatted location name
city: "Chicago"                         # City name
state: "Illinois"                       # State/province
country: "United States"                # Country

# Camera settings (if available)
iso: 100                                # ISO speed
exposure_time: 1/120                    # Shutter speed
f_number: 1.8                           # Aperture
focal_length: 4.2                       # Focal length in mm
---

This is a beautiful sunset over Lake Michigan in Chicago.
```

### Title Formatting

Photo titles are generated following these rules:

1. If the photo has a caption in iCloud, that caption is used as the title
2. If the photo has no caption, a title is generated in the format: "Photo taken on [Month Day, Year]"
   - Example: "Photo taken on July 15, 2023"
3. The date used is from EXIF data when available, or falls back to the photo's creation date

### Hugo Theme Integration

To display your photos in Hugo, you can use any theme that supports page bundles. Here's an example of a simple list template (`layouts/photostream/list.html`):

```html
{{ define "main" }}
  <h1>{{ .Title }}</h1>
  <div class="photo-grid">
    {{ range .Pages.ByDate.Reverse }}
      <div class="photo-item">
        <a href="{{ .RelPermalink }}">
          <img src="{{ .RelPermalink }}original.jpg" alt="{{ .Title }}" />
          <h2>{{ .Title }}</h2>
        </a>
      </div>
    {{ end }}
  </div>
{{ end }}
```

And a single photo template (`layouts/photostream/single.html`):

```html
{{ define "main" }}
  <article class="photo-page">
    <h1>{{ .Title }}</h1>
    
    <div class="photo-container">
      <img src="{{ .RelPermalink }}original.jpg" alt="{{ .Title }}" />
    </div>
    
    <div class="photo-metadata">
      {{ with .Params.camera_make }}
        <p><strong>Camera:</strong> {{ . }} {{ with $.Params.camera_model }}{{ . }}{{ end }}</p>
      {{ end }}
      
      {{ with .Params.exif_date }}
        <p><strong>Taken:</strong> {{ dateFormat "January 2, 2006" . }}</p>
      {{ end }}
      
      {{ with .Params.location }}
        <p><strong>Location:</strong> {{ . }}</p>
      {{ end }}
      
      {{ with .Params.iso }}
        <p><strong>Settings:</strong> ISO {{ . }}, 
          {{ with $.Params.exposure_time }}{{ . }}s, {{ end }}
          {{ with $.Params.f_number }}f/{{ . }}, {{ end }}
          {{ with $.Params.focal_length }}{{ . }}mm{{ end }}
        </p>
      {{ end }}
    </div>
    
    <div class="photo-content">
      {{ .Content }}
    </div>
  </article>
{{ end }}
```

## Troubleshooting

### Common Issues

**Problem**: Cannot find your iCloud shared album URL  
**Solution**: Make sure you're sharing the album publicly. In Photos, go to the album → Share → Share Link

**Problem**: No photos are downloaded  
**Solution**: Check that your album URL is correct and the album is publicly shared

**Problem**: Missing EXIF data  
**Solution**: Not all photos contain EXIF data. Photos that have been edited or sent through messaging apps often lose their EXIF information

**Problem**: Missing location data  
**Solution**: Not all photos contain GPS information. Check that location services were enabled when the photos were taken

### Verbose Logging

For more detailed debugging information, use the `RUST_LOG` environment variable:

```bash
# Informational logs
RUST_LOG=info icloudAlbum2hugo sync

# Debug level (more detailed)
RUST_LOG=debug icloudAlbum2hugo sync 

# Trace level (very verbose)
RUST_LOG=trace icloudAlbum2hugo sync
```

## Advanced Usage

### Cron Job for Automatic Updates

To set up automatic syncing, add a cron job:

```bash
# Edit crontab
crontab -e

# Add line to run sync daily at 2 AM
0 2 * * * cd /path/to/your/hugo/site && /path/to/icloudAlbum2hugo sync >> sync.log 2>&1
```

### Custom Hugo Page Paths

If you want to use a different directory structure than `content/photostream/<photo_id>`, you can modify the `out_dir` setting in your config.yaml:

```yaml
# Store photos in content/gallery instead
out_dir: "content/gallery"
```

## Development

Want to contribute? Great! Here's how to set up for development:

```bash
# Clone the repository
git clone https://github.com/harperreed/icloudAlbum2hugo.git
cd icloudAlbum2hugo

# Build and run with debug information
RUST_LOG=debug cargo run -- init

# Run tests
cargo test

# Run specific test
cargo test test_photo_title_formatting

# Run integration tests (requires iCloud token)
ICLOUD_TEST_TOKEN=YourToken cargo test --test icloud_integration_test -- --nocapture
```

## License

[MIT License](LICENSE)

## Credits

- Built with Rust 🦀
- Uses [kamadak-exif](https://github.com/kamadak/exif-rs) for EXIF parsing
- Uses [clap](https://github.com/clap-rs/clap) for command-line argument parsing
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP requests
- Uses [serde](https://github.com/serde-rs/serde) for serialization
- Created by [Harper Reed](https://github.com/harperreed)