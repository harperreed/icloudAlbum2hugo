# icloud2hugo

A command-line tool that syncs photos from iCloud Shared Albums to a Hugo site.

This tool fetches photos from a shared iCloud album, extracts EXIF data, performs reverse geocoding (when location data is available), and organizes everything into Hugo page bundles under `content/photostream/<photo_id>/`.

## Features

- Downloads new/updated photos at full resolution
- Removes photos that no longer exist in the album
- Extracts EXIF metadata (camera info, date/time, location)
- Reverse geocoding and location fuzzing for privacy
- Creates Hugo page bundles with proper frontmatter
- Maintains a master YAML index file

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/icloud2hugo.git
cd icloud2hugo

# Build with cargo
cargo build --release

# Move the binary to a location in your PATH (optional)
cp target/release/icloud2hugo ~/.local/bin/
```

## Usage

### Initial Setup

```bash
# Initialize with default configuration
icloud2hugo init

# Edit the generated config.yaml file
# Replace the placeholder album URL with your actual iCloud shared album URL
```

The configuration file (`config.yaml`) contains the following settings:

```yaml
album_url: "https://www.icloud.com/sharedalbum/#..."  # Your iCloud shared album URL
out_dir: "content/photostream"                        # Output directory for Hugo page bundles
data_file: "data/photos/index.yaml"                   # Path to the photo index file
fuzz_meters: 100.0                                    # Amount to fuzz GPS coordinates for privacy
```

### Syncing Photos

```bash
# Sync photos from the iCloud album to your Hugo site
icloud2hugo sync

# Use a custom config file
icloud2hugo sync --config /path/to/custom-config.yaml
```

During synchronization, the tool will:
1. Download new photos not yet in your local index
2. Update existing photos if they've changed in the remote album
3. Remove local photos that no longer exist in the remote album
4. Extract EXIF data and perform reverse geocoding
5. Create Hugo page bundles with the photos and metadata

### Checking Status

```bash
# Check the status of your local photos vs. the remote album
icloud2hugo status
```

The status command shows:
- How many photos are in sync
- How many new photos are available to download
- How many photos need to be updated
- How many photos will be removed
- Statistics about EXIF and location data

## Hugo Integration

The tool creates Hugo page bundles for each photo with the following structure:

```
content/
└── photostream/
    └── photo123456/
        ├── index.md   # Frontmatter with photo metadata
        └── original.jpg
```

The frontmatter in each `index.md` file includes:

```yaml
---
title: "Photo Title or Filename"
date: 2023-06-15T14:30:22+0000
guid: "photo123456"
original_filename: "IMG_1234.jpg"
width: 4032
height: 3024
camera_make: "Apple"
camera_model: "iPhone 12 Pro"
exif_date: 2023-06-15T14:30:22+0000
original_latitude: 41.878765
original_longitude: -87.635987
latitude: 41.878901
longitude: -87.636123
location: "Chicago, IL, USA"
city: "Chicago"
state: "Illinois"
country: "United States"
iso: 100
exposure_time: 1/120
f_number: 1.8
focal_length: 4.2
---

Photo caption or empty
```

## Demo Workflow

Here's a complete example workflow:

```bash
# 1. Initialize configuration
icloud2hugo init

# Edit config.yaml to set your iCloud shared album URL
# For example:
#   album_url: "https://www.icloud.com/sharedalbum/#B0aGWZmrRGZRiRW"
#   out_dir: "content/photostream"
#   data_file: "data/photos/index.yaml"
#   fuzz_meters: 100.0

# 2. Check status to see what will be synced
icloud2hugo status

# 3. Sync photos from iCloud to your Hugo site
icloud2hugo sync

# 4. Check status again to confirm everything is in sync
icloud2hugo status
```

## License

MIT

## Credits

Built with Rust 🦀 using libraries like kamadak-exif, clap, reqwest, and more.