Here we go:

---

## 🧾 Project Spec: `icloud2hugo`

### Overview

`icloud2hugo` is a Rust CLI tool that syncs a **public iCloud Photo Sharing album** into **Hugo-compatible page bundles**, maintaining a full-fidelity archive of images and metadata. It supports incremental syncs, deletions, and location fuzzing.

---

### 📦 Project Layout

```
project/
├── src/
│   ├── main.rs
│   ├── commands/       # init.rs, sync.rs, status.rs
│   ├── icloud.rs       # album parsing & crawling
│   ├── photo.rs        # image download, jhead parsing, checksum
│   ├── metadata.rs     # EXIF/gps parsing, fuzzing, reverse geocode
│   └── hugo.rs         # writes bundles, YAML index
├── config.yaml         # user config (album URL, output paths, etc)
├── content/photostream/photo-id/  # Hugo page bundles
└── data/photos/index.yaml         # Global photo index
```

---

### ⚙️ CLI Design

```bash
icloud2hugo init [--config config.yaml]
```

* Creates a template `config.yaml`
* Example fields:

  ```yaml
  icloud_album_url: "https://www.icloud.com/sharedalbum/#A1a2b3c4d5e6f7"
  content_dir: "content/photostream"
  data_index_path: "data/photos/index.yaml"
  fuzz_meters: 100
  ```

---

```bash
icloud2hugo sync [--config config.yaml] [--force]
```

* Fetches photo metadata from iCloud album (parsing exposed JS or JSON)
* Compares with local photo frontmatter + index.yaml
* Downloads new or changed photos
* Deletes page bundles for removed photos
* Extracts metadata via `jhead` (via Rust FFI or native Rust EXIF lib)
* Fuzzes GPS (e.g., Gaussian blur ±50–150m)
* Reverse-geocodes to city/state/country (using something like OpenCage API or Pelias)
* Writes `index.md` with frontmatter per photo
* Updates `data/photos/index.yaml`

---

```bash
icloud2hugo status [--config config.yaml]
```

* Summarizes:

  * ✅ Total photos
  * ➕ New to be added
  * 🔄 Changed metadata
  * ❌ To be deleted
  * 🛑 Errors or failed fetches

---

### 🧠 Smart Bits

* **Photo ID tracking** via iCloud GUID
* **Checksum-based change detection** (SHA-256 of image content)
* **Frontmatter schema** stored per bundle
* **Fuzzy GPS with original/safe split**
* **Optional reverse geocode caching** to avoid API hits
* **YAML master index** for Hugo list templates

---

### ⛓ Dependencies

Rust crates (tentative):

* `clap` – CLI args
* `serde_yaml`, `serde_json`, `serde` – config + index
* `reqwest` – album crawling, downloading
* `image` or `exif` – EXIF parsing (if avoiding jhead)
* `geo` or `geoutils` – fuzzing GPS
* `sha2` – checksums
* `rayon` – parallel sync
* optional: FFI wrapper for `jhead`, or native EXIF fallback

---

Want me to scaffold out `main.rs` + command modules next? Or focus on one (like `sync`)?

