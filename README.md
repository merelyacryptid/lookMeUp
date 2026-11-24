# LookMeUp

> A distraction-free, touch-friendly desktop reader with instantaneous, 100% offline dictionary lookups.

---

## About The Project

As someone who reads a lot on laptops and 2-in-1 touchscreen devices, I hated having my reading flow broken every time I ran into an unfamiliar word without an active internet connection (which happens constantly around my college campus). Most online dictionary tools fail without Wi-Fi, leak reading activity to remote APIs, or clutter the screen with competing popups.

LookMeUp is built as a distraction-free, touch-friendly reading environment:
* **Touch-Screen Friendly:** No keyboard needed. Tap, double-tap, or highlight any word on your touchscreen or with your mouse to instantly see its meaning.
* **Single Unified Popup:** Because LookMeUp hosts the reading canvas, you get exactly one clean, beautiful dictionary bubble positioned directly over the tapped word.
* **Instant Offline SQLite (<1 ms):** Directly queries a local, pre-indexed SQLite B-tree database with 224,954 headwords and 218,157 morphological lemma fallbacks (e.g. tapping "epiphanies" routes to "epiphany", "running" to "run").
* **Word Bank & Frequency Tracker:** Automatically tracks your vocabulary discoveries in local storage (`user_data.db`). See how many times you have encountered a word (`2x`, `3x`), save words with one tap, and search through your personal Word Bank drawer anytime.
* **Typography Controls & Image-Inlined EPUB Engine:** Native pure-Rust EPUB extractor that inlines book illustrations as Base64 Data URIs, complete with customizable font styles, text sizing, line/word/paragraph spacing, and text alignment.
* **Complete Privacy:** 100% offline. Zero network calls, zero telemetry, zero accounts.

---

## Architecture

```
+-------------------------------------------------------------+
|              LookMeUp In-Reader Canvas (Webview)            |
|                                                             |
|   - Native pure-Rust EPUB extractor & Base64 image inliner  |
|   - Direct-DOM touch text selection (window.getSelection)   |
|   - Single inline dictionary card (range.getBoundingClientRect)
|   - Word Bank drawer with frequency counts & saved words    |
|   - Typography customizer popover (fonts, sizing, spacing)  |
+------------------------------+------------------------------+
                               |
               Tauri IPC (invoke / JSON)
                               |
+------------------------------v------------------------------+
|                     Tauri v2 Rust Core                      |
|                                                             |
|   - Instant exact-match B-tree query (<1 ms)                |
|   - Morphological lemma fallback (e.g. "epiphanies" -> "epiphany")
|   - Device-first user telemetry via SQLite UPSERT           |
|   - Pure-Rust ZIP extraction and Base64 Data URI inlining   |
+------------------------------+------------------------------+
                               |
            Embedded SQLite Databases (Local Disk)
                               |
       +-----------------------+-----------------------+
       |                                               |
+------v-----------------------+ +---------------------v------+
|       dictionary.db          | |       user_data.db         |
|  (Read-Only Reference Data)  | |    (Mutable User Store)    |
| - 224,954 headwords (entries)| | - Word Bank history        |
| - 218,157 morphology mappings| | - Device lookup frequency  |
| - B-tree indexed (NOCASE)    | | - Explicit saved words     |
+------------------------------+ +----------------------------+
```

---

## Tech Stack

* **Core Runtime:** Tauri v2 (Rust 2024 edition)
* **Backend:** Rust, `rusqlite` (bundled SQLite3), `serde` / `serde_json`, `zip` (pure-Rust extractor), `base64`
* **Plugins:** `tauri-plugin-dialog`, `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`
* **Frontend:** Vanilla TypeScript, Vite, CSS3 (reader canvas, glassmorphic drawers, typography popover)
* **Data Pipeline:** Python script compiling 224,954 Webster's Unabridged and Open English WordNet entries into an indexed, sub-millisecond SQLite database (`dictionary.db`) with 218,157 morphological lemma transformations.

---

## Dictionary Pipeline & Data Sources

LookMeUp bundles a comprehensive, 100% offline dictionary database (`dictionary.db`) featuring over 224,000 headwords and 218,000 morphological lemma mappings derived from public domain Webster's Revised Unabridged Dictionary (1913) and Open English WordNet.

To rebuild or refresh the offline dictionary from source:
```bash
python scripts/build_full_db.py
```

### High-Throughput Ingestion Architecture
* **Pragmas:** Utilizes `PRAGMA synchronous = OFF;`, `PRAGMA journal_mode = MEMORY;`, and `PRAGMA cache_size = 100000;` to achieve insert speeds exceeding 50,000 records/sec.
* **Deferred B-Tree Indexing:** Tables are loaded raw before creating `COLLATE NOCASE` B-tree indexes, converting repetitive index rebalances into a single fast post-sort.
* **Morphological Fallback Engine:** Regular English inflections (`-s`, `-es`, `-ies`, `-ed`, `-d`, `-ing`) and irregular forms (`ran` -> `run`, `children` -> `child`, etc.) are mapped into the `forms` index for instant fallback whenever an inflected form is selected.

---

## Roadmap

- [x] **Phase 0: Environment & Scaffolding** - Toolchain setup, Tauri v2 scaffold, folder anatomy.
- [x] **Phase 1: IPC Bridge** - Rust `lookup_word` command, typed definitions, serde serialization, and frontend `invoke`.
- [x] **Phase 2: Offline SQLite Dictionary** - SQLite schema, Python build script, exact match, and morphological lemma fallback in Rust.
- [x] **Phase 3: Global Shortcut & Text Capture** - System-wide hotkey listener (`Ctrl+Alt+D`) and active word capture.
- [x] **Phase 4: Floating Cursor Popup** - Frameless, auto-positioning window at mouse cursor with blur/Esc auto-dismiss.
- [x] **Phase 5: Full 225,000+ Word Dictionary Pipeline** - High-throughput batch ingestion pipeline compiling Webster's 1913 and Open English WordNet into an indexed B-tree database with 218,000+ morphological mappings (<1 ms latency).
- [x] **Phase 6.1: In-Reader EPUB Canvas & Typography** - Pure-Rust EPUB extraction, Base64 image inlining, Table of Contents, and typography controls.
- [x] **Phase 6.2: Word Bank & Device-First Lookup Frequency** - Local SQLite vocabulary tracking (`user_data.db`), UPSERT counting, badges, and searchable Word Bank drawer.
- [x] **Phase 6.3: Explicit Save Word & Vocabulary Management** - Prominent "Save Word" toggle on definition popups, `is_saved` SQLite migration, and dedicated "All" vs "Saved" Word Bank filter tabs.
- [ ] **Phase 7: EPUB Library & Favorites Grid** - Local cover-art library grid, "Read" status, and favorites collection.
- [ ] **Phase 8: Highlights & Annotations** - In-reader text highlighting and saved notes drawer.

---

## License

MIT
