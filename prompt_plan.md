Below is a **multi-stage plan** for building this Rust-based “icloud2hugo” tool. After outlining the **blueprint**, you’ll see it broken down into **iterative chunks**, then **refined** into small, testable steps. Finally, there’s a set of **LLM prompts** you can feed into a code-generation system for incremental, test-driven coding—each prompt references and builds upon the previous steps.

---

## 1. Detailed Blueprint

### 1.1 Goals

* Command-line tool, `icloud2hugo`, with subcommands: `init`, `sync`, `status`.
* Sync a public iCloud Photo Sharing album:

  * Download new/updated photos at full resolution.
  * Remove photos that no longer exist in the album.
  * Use EXIF/jhead to parse metadata.
  * Reverse geocode & fuzz location.
  * Write Hugo page bundles under `content/photostream/<photo_id>/`.
  * Maintain a master YAML file in `data/photos/index.yaml`.
* Provide a minimal config file for base settings (iCloud album URL, etc.).

### 1.2 Architecture

* **CLI**:

  * Built using [Clap](https://docs.rs/clap/latest/clap/).
  * Subcommands:

    * `init`: Create a default `config.yaml`.
    * `sync`: Perform add/update/delete operations.
    * `status`: Summarize or show local vs. remote differences.
* **Data/Config**:

  * Store config in `config.yaml` (with an optional override path).
  * `data/photos/index.yaml` for a single global listing of known photos.
  * Frontmatter in each `content/photostream/<id>/index.md`.
* **Sync Logic**:

  1. **Fetch** iCloud shared album JSON (public endpoint).
  2. **Compare** with local metadata (using `index.yaml` or frontmatter).
  3. **Download** new/updated items:

     * Store as `original.jpg` in each page bundle.
     * Generate or update `index.md`.
  4. **Delete** removed items from the local directory structure.
  5. **EXIF** parse:

     * Extract camera info, time, GPS, etc.
  6. **Reverse Geocode** & **Fuzz** GPS:

     * Store both `original` and `safe` lat/lon in frontmatter.
     * Possibly use an external API or offline data for geocoding.
  7. **Update** `data/photos/index.yaml`.
* **Testing**:

  * Use `cargo test` with small mock scenarios:

    * Mock iCloud responses.
    * Check correct file creation, updates, deletions.
    * Check metadata parsing and frontmatter correctness.

### 1.3 Implementation Overview

1. **Project Scaffold**: Cargo init, Clap dependencies, directories, placeholders.
2. **CLI & Config**: Implement subcommands, load/save config.
3. **Core Sync**:

   * iCloud fetch code.
   * Data comparison logic.
   * Photo download & local storage.
   * Update master YAML.
4. **EXIF & Reverse Geocode**:

   * Integrate `exif` or shell out to `jhead`.
   * Use a chosen geocoding approach.
   * Fuzz location.
5. **Integrate Everything**:

   * Ensure TDD coverage.
   * Final polish, unify code, refactor.

---

## 2. Break the Blueprint into Iterative Chunks

1. **Chunk 1**: Project Setup

   * Initialize cargo project, add minimal CLI with Clap.
   * “Hello world” subcommands: `init`, `sync`, `status`.

2. **Chunk 2**: Config Management

   * Implement `init` to generate `config.yaml` with defaults.
   * Add code to load config from file or fallback to CLI argument.

3. **Chunk 3**: Basic iCloud Fetch

   * Implement a function to fetch & parse iCloud JSON from the shared album link.
   * Add minimal test with a mock JSON response.

4. **Chunk 4**: Local Data Structure (`index.yaml`)

   * Write basic read/write logic for `data/photos/index.yaml`.
   * Keep track of known photos (IDs, checksums, timestamps, etc.).

5. **Chunk 5**: Sync Logic (Add/Update)

   * Compare remote iCloud photo list to local `index.yaml`.
   * Download new/updated images into `content/photostream/<id>/`.
   * Create or update frontmatter in `index.md`.
   * Write tests for add/update steps.

6. **Chunk 6**: Sync Logic (Delete)

   * Identify removed photos.
   * Remove the corresponding local directories.
   * Update `index.yaml` accordingly.
   * Test removal flow.

7. **Chunk 7**: EXIF Parsing

   * Integrate a Rust EXIF library or shell out to `jhead`.
   * Populate camera info, dates, etc. in frontmatter.
   * Test with sample EXIF data.

8. **Chunk 8**: Reverse Geocode & Fuzz

   * Implement location geocoding call (mock or real).
   * Add random offset to “safe” coordinates.
   * Store “original” vs. “safe” in frontmatter.
   * Test with known lat/lon and check for fuzzed result.

9. **Chunk 9**: `status` Subcommand

   * Summarize local vs. remote (new, changed, deleted).
   * Possibly show config or counts.

10. **Chunk 10**: Final Integration & Polishing

    * Validate edge cases (no photos, offline, album changed).
    * Ensure TDD coverage.
    * Optimize error handling & logs.

---

## 3. Refine Chunks into Smaller Steps

We’ll now break each chunk into **smaller steps** that can be tackled in a single session/prompt. The goal is to keep tasks bite-sized but impactful.

### Chunk 1: Project Setup

1. **Create Cargo Project**: `cargo new icloud2hugo`.
2. **Add Dependencies**:

   * `clap = "4.x"`
   * `serde`, `serde_yaml`, `serde_json`
   * `reqwest`
   * `tokio` (if we plan async)
   * `anyhow` or `eyre` for error handling
3. **Create Basic CLI Structure**:

   * `init`, `sync`, `status` subcommands as stubs.
   * `main()` with Clap parse.

### Chunk 2: Config Management

1. **`init` Command**:

   * Write out a default `config.yaml` with placeholders:

     ```yaml
     album_url: "https://www.icloud.com/sharedalbum/#..."
     out_dir: "content/photostream"
     data_file: "data/photos/index.yaml"
     ```
2. **Load Config**:

   * Implement a function to read a user-specified config path (or fallback).
   * Validate required fields (album\_url, etc.).

### Chunk 3: Basic iCloud Fetch

1. **Create a Struct** to model iCloud JSON data (photo objects, IDs, etc.).
2. **Fetch** the iCloud JSON:

   * Implement a function `fetch_icloud_metadata(&str) -> Result<ICloudAlbum, Error>`.
   * Use `reqwest` to GET the album URL, parse JSON.
3. **Mock** or **Test** with a sample JSON response.

### Chunk 4: Local Data (`index.yaml`)

1. **Create a Struct** for local photo metadata (id, filename, etc.).
2. **Read** existing `index.yaml` (if any).
3. **Write** updated `index.yaml` after changes.
4. **Test** read/write flow.

### Chunk 5: Sync Logic (Add/Update)

1. **Compare** remote list vs. local index.
2. For **new photos**:

   * Create a new folder under `content/photostream/<photo_id>`.
   * Download the photo as `original.jpg`.
   * Create a minimal `index.md` with frontmatter (title, date, etc.).
   * Update `index.yaml`.
3. For **updated photos**:

   * Re-download if needed (checksum mismatch or forced).
   * Update frontmatter and `index.yaml`.
4. **Test** with mock data (some new, some old).

### Chunk 6: Sync Logic (Delete)

1. Identify photos in `index.yaml` not present in iCloud metadata.
2. Remove associated folders under `content/photostream/<photo_id>`.
3. Remove from `index.yaml`.
4. Test with a mock scenario.

### Chunk 7: EXIF Parsing

1. Add an `exif` or `jhead` shell-out step after download.
2. Extract camera model, date/time, lat/lon, etc.
3. Update `index.md` frontmatter fields.
4. Test with sample images or mock EXIF data.

### Chunk 8: Reverse Geocode & Fuzz

1. Use a placeholder or real geocoding function:

   * e.g., `reverse_geocode(lat, lon) -> (city, state, country)`
2. Fuzz lat/lon:

   * `fuzzed_lat = lat + (random small offset)`
   * `fuzzed_lon = lon + (random small offset)`
3. Write `original` vs. `safe` coords to frontmatter.
4. Test with mock coordinates.

### Chunk 9: `status` Subcommand

1. Load remote iCloud metadata & local index.
2. Show counts: new photos, changed photos, removed photos.
3. Possibly show config details.
4. Test that it prints correct info.

### Chunk 10: Final Integration & Polishing

1. Review code for consistency & style.
2. Validate error handling for network, file IO, etc.
3. Add a final integration test.

---

## 4. Iterating Step Sizing

Reviewing the above:

* Each chunk is split into 2–4 smaller steps, which is fairly granular.
* Each step can be tackled in a single TDD session.
* This should be small enough to “build safely” but big enough to move the project forward.

If you find any step too large, you could break it further (e.g., “EXIF parsing for date/time only” vs. “EXIF parsing for all fields”). However, the above is typically workable.

---

## 5. Series of Prompts for a Code-Generation LLM

Below is a **sequence of TDD-style prompts**. Each prompt is written as **text** (in triple backticks). You’d feed these into your favorite code generator, one at a time, in order. Each prompt:

* References the code from previous steps.
* Asks for a minimal test + implementation approach.
* Ensures no orphan code.

### Prompt 1: Project Setup (COMPLETED)

```
You are building the "icloud2hugo" Rust CLI tool. Start by creating a new cargo project with a minimal CLI using Clap. 
- Project name: icloud2hugo
- Dependencies: clap = "4", anyhow, tokio (if async), etc.
- Subcommands: init, sync, status (stubs only).
- Provide tests that verify we can run each subcommand.

Output your solution with fully commented code. Only a brief explanation, as needed.
```

### Prompt 2: Config Management (COMPLETED)

```
Expand the existing project with config management. Implement:
1. A function to generate a default config.yaml in the current directory for the `init` subcommand.
2. A function to load config from a specified path or from a default path (if no path given).
3. Tests that ensure:
   - `init` creates a valid config.yaml with placeholders.
   - `sync` and `status` can load the config or fail gracefully if missing.
```

### Prompt 3: Basic iCloud Fetch

```
Now implement a function to fetch and parse the iCloud shared album JSON. 
1. Define a struct for the parsed album/photo metadata. 
2. Use `reqwest` to GET the album URL from the config. 
3. Parse JSON into your structs. 
4. Write unit tests with a mock HTTP server or dummy JSON to ensure parsing works.
5. Integrate a minimal call in `sync` to load config, fetch JSON, and print out the results (so we know it works).
```

### Prompt 4: Local Data Structure (index.yaml)

```
We need to store a local index of photos. 
1. Create a Rust struct representing our local index (photo_id, maybe checksums, date downloaded, etc.).
2. Implement read/write of `data/photos/index.yaml` using `serde_yaml`.
3. Write tests that confirm we can load, update, and save changes to the file.
4. Wire this into the `sync` subcommand so we can load existing data, or create it if it doesn't exist.
```

### Prompt 5: Sync Logic (Add/Update)

```
Implement the first half of the sync logic: 
1. Compare the remote iCloud metadata to our local index. 
2. For each new photo, create a directory under content/photostream/<photo_id>, 
   download original.jpg, and create a minimal index.md with frontmatter. 
3. For updated photos (if a checksum is different), re-download.
4. Update the local index.yaml accordingly.
5. Provide tests that mock iCloud data with some new photos, verify new directories are created, index.yaml is updated, etc.
```

### Prompt 6: Sync Logic (Delete)

```
Continue the sync logic: 
1. Identify photos in index.yaml that are no longer in the remote iCloud data. 
2. Remove their content/photostream/<photo_id> directories. 
3. Remove them from index.yaml. 
4. Test this with a scenario where a photo was removed from the mock remote data.
```

### Prompt 7: EXIF Parsing

```
Now add EXIF data extraction. 
1. Use a Rust EXIF crate or shell out to jhead after downloading the photo. 
2. Parse date/time, camera make/model, lat/lon, etc. 
3. Insert this into the frontmatter of index.md. 
4. Test using a sample image or a test double. 
5. Ensure the date/time is stored as a valid Hugo date in frontmatter.
```

### Prompt 8: Reverse Geocode & Fuzz

```
Implement location metadata:
1. For each photo with GPS data, call a function reverse_geocode(lat, lon) -> "Chicago, IL, USA" (mock or real). 
2. Fuzz lat/lon with a small random offset. 
3. Store both original and safe lat/lon in frontmatter. 
4. Test by mocking the geocode function, verifying that the frontmatter includes both original and fuzzed coords.
```

### Prompt 9: status Subcommand

```
Enhance the `status` subcommand:
1. Load remote iCloud data and local index. 
2. Print how many photos are new, changed, or removed. 
3. Possibly show a summary of config as well. 
4. Test with a variety of scenarios to confirm we display correct numbers.
```

### Prompt 10: Final Integration & Polishing

```
Finalize and refactor:
1. Polish error handling and logs for each subcommand. 
2. Ensure all tests pass, no dead code. 
3. Provide a short demonstration of the entire workflow from init -> sync -> status. 
4. Confirm the code is well structured and documented. 
5. Ship the final solution.
```

---

## Summary

Following this plan:

1. You have a **clear blueprint** for the entire tool.
2. It’s **chunked** into well-defined stages.
3. Each stage is **further broken down** into smaller testable steps.
4. A **series of LLM prompts** ensures incremental, test-driven development without leaving partial or orphan code behind.

You can now take these prompts (in the code fences) and feed them sequentially into a code-generation LLM, verifying the output and ensuring each step is tested before moving on. This approach will help you maintain best practices, incrementally build features, and produce a robust `icloud2hugo` tool.

