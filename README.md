# icloudAlbum2hugo

A command-line tool that syncs photos from iCloud Shared Albums to a Hugo site.

This tool fetches photos from a shared iCloud album, extracts EXIF data, performs reverse geocoding (when location data is available), and organizes everything into Hugo page bundles. It supports two output modes: **photostream** (individual page bundles per photo) and **gallery** (single page bundle with all photos).

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
- 🖼️ Multiple output modes: photostream (individual pages) or gallery (single page)
- 🔒 Privacy-focused features with configurable frontmatter options
- 🆔 UUID generation for unique gallery identification

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Detailed Usage](#detailed-usage)
  - [Command: init](#command-init)
  - [Command: sync](#command-sync)
  - [Command: status](#command-status)
- [Configuration Options](#configuration-options)
  - [Photostream Configuration](#photostream-configuration)
  - [Gallery Configuration](#gallery-configuration)
  - [Privacy Settings](#privacy-settings)
- [Hugo Integration](#hugo-integration)
  - [Photostream Mode](#photostream-mode)
  - [Gallery Mode](#gallery-mode)
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

The configuration file supports multiple output modes and privacy settings. icloudAlbum2hugo uses a modern multi-output configuration format that allows you to sync from multiple albums into different locations with different settings.

### Photostream Configuration

For individual photo page bundles (traditional mode):

```yaml
# Global settings
fuzz_meters: 100.0  # Distance in meters to fuzz location data for privacy

# Output configurations
outputs:
  - output_type: photostream
    album_url: "https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW"
    out_dir: "content/photostream"
    data_file: "data/photos/photostream.yaml"
    name: "My Photo Stream"  # Optional: custom name for this output
    description: "Daily photo updates"  # Optional
    enabled: true
```

### Gallery Configuration

For single-page galleries with all photos:

```yaml
# Global settings
fuzz_meters: 100.0

# Output configurations
outputs:
  - output_type: gallery
    album_url: "https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW"
    out_dir: "content/galleries/vacation"
    data_file: "data/photos/vacation.yaml"
    name: "Summer Vacation 2023"  # Optional: will use album name if not provided
    description: "Photos from our amazing summer trip"  # Optional
    enabled: true
```

### Multiple Outputs

You can configure multiple outputs to sync different albums to different locations:

```yaml
fuzz_meters: 100.0

outputs:
  # Photostream for daily photos
  - output_type: photostream
    album_url: "https://www.icloud.com/sharedalbum/#DailyPhotos123"
    out_dir: "content/photostream"
    data_file: "data/photos/daily.yaml"
    name: "Daily Photos"
    enabled: true

  # Gallery for vacation photos
  - output_type: gallery
    album_url: "https://www.icloud.com/sharedalbum/#VacationAlbum456"
    out_dir: "content/galleries/vacation-2023"
    data_file: "data/photos/vacation.yaml"
    name: "Summer Vacation 2023"
    description: "Our amazing trip to Europe"
    enabled: true

  # Gallery for family events
  - output_type: gallery
    album_url: "https://www.icloud.com/sharedalbum/#FamilyEvents789"
    out_dir: "content/galleries/family-events"
    data_file: "data/photos/family.yaml"
    name: "Family Events"
    enabled: true
```

### Privacy Settings

Each output can include privacy configuration for enhanced control over Hugo frontmatter:

```yaml
outputs:
  - output_type: gallery
    album_url: "https://www.icloud.com/sharedalbum/#PrivateAlbum123"
    out_dir: "content/galleries/private-moments"
    data_file: "data/photos/private.yaml"
    name: "Private Family Moments"
    privacy:
      nofeed: true          # Exclude from RSS feeds
      noindex: true         # Exclude from search engine indexing
      uuid_slug: true       # Use UUID-based slugs instead of readable names
      unlisted: true        # Mark as unlisted (not shown in listings)
      robots_noindex: true  # Add robots meta tag with noindex,nofollow
```

### Legacy Configuration

The tool also supports legacy single-output configuration for backward compatibility:

```yaml
album_url: "https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW"
out_dir: "content/photostream"
data_file: "data/photos/index.yaml"
fuzz_meters: 100.0
```

### Finding Your iCloud Shared Album URL

1. In the iCloud Photos app or iCloud.com, navigate to the shared album
2. Click on the "Share" button
3. Select "Copy Link"
4. Paste this URL into your config.yaml file

The URL should look like: `https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW`

## Hugo Integration

The tool supports two output modes, each creating different Hugo site structures optimized for different use cases.

### Photostream Mode

Creates individual page bundles for each photo, ideal for blog-style photo posts and detailed photo pages.

#### Directory Structure

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
        └── photostream.yaml  # Master index of all photos
```

### Gallery Mode

Creates a single page bundle containing all photos from an album, ideal for photo galleries and collections.

#### Directory Structure

```text
your-hugo-site/
├── config.yaml            # Your Hugo config
├── content/
│   └── galleries/
│       └── vacation-2023/    # Gallery page bundle
│           ├── index.md      # Gallery frontmatter with photo list
│           ├── photo123.jpg  # Individual photo files
│           ├── photo456.jpg
│           └── photo789.jpg
└── data/
    └── photos/
        └── vacation.yaml     # Master index with gallery info
```

### Frontmatter Fields

#### Photostream Frontmatter

Each photostream `index.md` file contains comprehensive frontmatter:

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

#### Gallery Frontmatter

Gallery `index.md` files contain gallery-specific frontmatter with photo listings:

```yaml
---
title: "Summer Vacation 2023"
date: 2023-07-15T14:30:22+0000
type: gallery
layout: gallery
uuid: "550e8400-e29b-41d4-a716-446655440000"    # Unique gallery identifier
description: "Photos from our amazing summer trip"
photo_count: 25

# Privacy settings (when configured)
nofeed: true                                     # Exclude from RSS feeds
noindex: true                                    # Exclude from search indexing
slug: "550e8400-e29b-41d4-a716-446655440000"     # UUID-based slug for privacy
unlisted: true                                   # Mark as unlisted
robots: "noindex,nofollow"                       # Robots meta tag

# Photo list with metadata
photos:
  - filename: photo123.jpg
    caption: "July 15, 2023 • Chicago, IL, USA • Apple iPhone 12 Pro"
    mime_type: "image/jpeg"
    original_caption: "Beautiful sunset over the lake"
    location: "Chicago, IL, USA"
    camera_make: "Apple"
    camera_model: "iPhone 12 Pro"
    date: 2023-07-15T14:30:22+0000
  - filename: photo456.jpg
    caption: "July 16, 2023 • Milwaukee, WI, USA"
    mime_type: "image/jpeg"
    location: "Milwaukee, WI, USA"
    date: 2023-07-16T10:15:00+0000
---

Our summer vacation was amazing! Here are some of the highlights from our trip through the Great Lakes region.
```

### Title Formatting

Photo titles are generated following these rules:

1. If the photo has a caption in iCloud, that caption is used as the title
2. If the photo has no caption, a title is generated in the format: "Photo taken on [Month Day, Year]"
   - Example: "Photo taken on July 15, 2023"
3. The date used is from EXIF data when available, or falls back to the photo's creation date

### Hugo Theme Integration

To display your photos in Hugo, you can use any theme that supports page bundles. Below are examples for both photostream and gallery modes.

#### Photostream Templates

For individual photo pages, create these templates:

**Photostream List Template** (`layouts/photostream/list.html`):

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

**Photostream Single Template** (`layouts/photostream/single.html`):

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

#### Gallery Templates

For gallery pages that display all photos from an album:

**Gallery List Template** (`layouts/gallery/list.html`):

```html
{{ define "main" }}
  <h1>{{ .Title }}</h1>
  <div class="gallery-grid">
    {{ range .Pages.ByDate.Reverse }}
      <div class="gallery-item">
        <a href="{{ .RelPermalink }}">
          {{ with .Params.photos }}
            {{ $firstPhoto := index . 0 }}
            <img src="{{ $.RelPermalink }}{{ $firstPhoto.filename }}" alt="{{ $.Title }}" />
          {{ end }}
          <h2>{{ .Title }}</h2>
          <p>{{ .Params.photo_count }} photos</p>
          {{ with .Params.description }}
            <p class="description">{{ . }}</p>
          {{ end }}
        </a>
      </div>
    {{ end }}
  </div>
{{ end }}
```

**Gallery Single Template** (`layouts/gallery/single.html`):

```html
{{ define "main" }}
  <article class="gallery-page">
    <header class="gallery-header">
      <h1>{{ .Title }}</h1>
      {{ with .Params.description }}
        <p class="gallery-description">{{ . }}</p>
      {{ end }}
      <p class="gallery-info">{{ .Params.photo_count }} photos</p>
    </header>

    <div class="gallery-content">
      {{ .Content }}
    </div>

    <div class="photo-grid">
      {{ range .Params.photos }}
        <div class="photo-item" data-date="{{ .date }}" data-location="{{ .location }}">
          <img src="{{ $.RelPermalink }}{{ .filename }}"
               alt="{{ .caption }}"
               title="{{ .caption }}" />

          <div class="photo-caption">
            {{ with .original_caption }}
              <p class="original-caption">{{ . }}</p>
            {{ end }}

            <div class="photo-meta">
              {{ with .location }}
                <span class="location">📍 {{ . }}</span>
              {{ end }}

              {{ with .camera_make }}
                <span class="camera">📷 {{ . }}{{ with $.camera_model }} {{ . }}{{ end }}</span>
              {{ end }}

              <span class="date">📅 {{ dateFormat "January 2, 2006" .date }}</span>
            </div>
          </div>
        </div>
      {{ end }}
    </div>
  </article>
{{ end }}
```

**Gallery CSS Example** (`static/css/gallery.css`):

```css
.gallery-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 2rem;
  margin: 2rem 0;
}

.gallery-item {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
  transition: transform 0.2s ease;
}

.gallery-item:hover {
  transform: translateY(-4px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}

.photo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 1rem;
  margin: 2rem 0;
}

.photo-item {
  position: relative;
  overflow: hidden;
  border-radius: 8px;
  background: #f9f9f9;
}

.photo-item img {
  width: 100%;
  height: auto;
  display: block;
  transition: transform 0.3s ease;
}

.photo-item:hover img {
  transform: scale(1.05);
}

.photo-caption {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(transparent, rgba(0,0,0,0.8));
  color: white;
  padding: 1rem;
  transform: translateY(100%);
  transition: transform 0.3s ease;
}

.photo-item:hover .photo-caption {
  transform: translateY(0);
}

.photo-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  font-size: 0.8rem;
  margin-top: 0.5rem;
}

.photo-meta span {
  background: rgba(255,255,255,0.2);
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
}
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
