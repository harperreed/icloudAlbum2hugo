**Project Spec**

1. **Name**: `icloud2hugo`
2. **Purpose**:

   * Crawl a public iCloud Photo Sharing album.
   * Fetch full-resolution photos, parse EXIF metadata (via jhead or a Rust library), and build [Hugo](https://gohugo.io/) page bundles.
   * Create or update `content/photostream/<photo_id>/index.md` + `original.jpg`.
   * Maintain a global `data/photos/index.yaml` with aggregated metadata.
   * Reverse geocode location, then store both original & “fuzzed” GPS in frontmatter.
   * Track deletes and removals (sync = add, update, delete).
3. **Workflow**:

   1. **init** – Create a default `config.yaml`.
   2. **sync** –

      * Read album metadata JSON from iCloud.
      * Compare to local records (frontmatter, `data/photos/index.yaml`).
      * Download new or changed photos.
      * Remove local items missing from the online album.
      * Extract EXIF.
      * Reverse geocode + fuzz.
      * Write or update each page bundle.
      * Update `data/photos/index.yaml`.
   3. **status** – Summarize what’s changed, what’s missing, etc.
4. **Data Storage**:

   * `content/photostream/<photo_id>/index.md` with frontmatter.
   * `data/photos/index.yaml` for a global listing (photo IDs, metadata).
   * Possibly store checksums, sync timestamps, etc. in frontmatter or the YAML index.
5. **Dependencies**:

   * **`reqwest`** or similar for HTTP.
   * **`serde`, `serde_json`, `serde_yaml`** for reading/writing metadata.
   * **`clap`** for CLI.
   * **`anyhow`** (or `eyre`) for error handling.
   * **`exif`** crate or shell out to `jhead` for metadata.
   * **`chrono`** for date/time.
   * **`geocoding`** or an external API for reverse geocoding.
6. **Sync Strategy**:

   * Maintain a stable photo ID from iCloud to detect add/remove.
   * If photos can change, use a `--force` or a checksum check to re-download.
   * Optionally fuzz GPS with some random offset each run or store a single stable offset per photo.

---

**Sample CLI Scaffolding (Rust)**

```rust
use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

/// icloud2hugo: Simple tool to sync photos from an iCloud shared album into a Hugo site.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct ICloud2Hugo {
    /// Path to the config file (e.g., ./config.yaml)
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize default config file
    Init,
    /// Sync (download, update, delete) photos
    Sync,
    /// Show sync status
    Status,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = ICloud2Hugo::parse();

    match cli.command {
        Commands::Init => {
            // 1. Create a default config.yaml
            // 2. Possibly prompt or just write a standard scaffold
            println!("Initializing config...");
            // ...
        }
        Commands::Sync => {
            // 1. Load config (CLI or default)
            // 2. Fetch iCloud album JSON
            // 3. Compare with local data
            // 4. Download new photos, remove deleted
            // 5. Extract EXIF
            // 6. Reverse geocode + fuzz
            // 7. Write frontmatter, update data/photos/index.yaml
            println!("Syncing photos...");
            // ...
        }
        Commands::Status => {
            // 1. Read local data
            // 2. Summarize what's new, changed, or missing
            println!("Album status:");
            // ...
        }
    }

    Ok(())
}
```

**Brief Explanation**:

* `clap` handles CLI parsing.
* `ICloud2Hugo` is our main struct for CLI args.
* `Commands` enumerates subcommands.
* Each subcommand implements the steps for init, sync, or status.

**Alternative**:

* Instead of `clap`, use [structopt](https://docs.rs/structopt) (older style but simpler for some).
* Use a simple `.toml` or `.json` for config.
* Shell out to `jhead` for metadata extraction instead of a Rust EXIF crate if you prefer.
