# LookMeUp

> A distraction-free, touch-friendly desktop reader with instantaneous, 100% offline dictionary lookups.

---

## About The Project

As someone who reads a lot on laptops and 2-in-1 touchscreen devices, I hated having my reading flow broken every time I ran into an unfamiliar word without an active internet connection (which happens constantly around my college campus). Most online dictionary tools fail without Wi-Fi, leak reading activity to remote APIs, or clutter the screen with competing popups.

LookMeUp is built as a distraction-free, touch-friendly reading environment:
* **Touch-Screen Friendly:** No keyboard needed. Tap, double-tap, or highlight any word on your touchscreen or with your mouse to instantly see its meaning.
* **Single Unified Popup:** Because LookMeUp hosts the reading canvas, you get exactly one clean, beautiful dictionary bubble positioned directly over the tapped word.
* **Instant Offline SQLite (<50 ms):** Directly queries a local, pre-indexed SQLite B-tree database with morphological lemma fallback (e.g. tapping "scored" routes to "score").
* **Word Bank & Frequency Tracker:** Automatically tracks your vocabulary discoveries in local storage (`user_data.db`). See how many times you have encountered a word (`2x`, `3x`) and search through your personal Word Bank drawer anytime.
* **Typography Controls & Image-Inlined EPUB Engine:** Native pure-Rust EPUB extractor that inlines book illustrations as Base64 Data URIs, complete with customizable font styles, text sizing, line/word/paragraph spacing, and text alignment.
* **Complete Privacy:** 100% offline. Zero network calls, zero telemetry, zero accounts.

---

## Architecture

```
+-------------------------------------------------------------+
|        Any Application (Kindle, Sumatra, Calibre, Chrome)   |
|                                                             |
|   1. User selects word and presses Global Hotkey            |
+------------------------------+------------------------------+
                               | OS Selection / Clipboard
+------------------------------v------------------------------+
|                     Tauri v2 Rust Core                      |
|                                                             |
|   - Global shortcut listener (tauri-plugin-global-shortcut) |
|   - Instant exact match & lemma fallback query (rusqlite)   |
|   - Cursor coordinate detection & floating window positioning|
+------------------------------+------------------------------+
                               |
               Tauri IPC (invoke / JSON)
                               |
+------------------------------v------------------------------+
|            Floating Definition Popup (Webview)              |
|                                                             |
|   - Frameless, translucent dark-mode card near cursor       |
|   - Sub-50ms render with vanilla TypeScript & CSS           |
|   - Auto-hides on Esc or blur                               |
+-------------------------------------------------------------+
```

---

## Tech Stack

* **Core Runtime:** Tauri v2 (Rust 2024 edition)
* **Backend:** Rust, `rusqlite` (bundled SQLite3), `serde` / `serde_json`
* **Plugins:** `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`
* **Frontend:** Vanilla TypeScript, Vite, CSS3 (frameless floating card UI)
* **Data Pipeline:** Python script converting Kaikki.org Wiktionary JSON dumps into compact, indexed SQLite databases

---

## Roadmap

- [x] **Phase 0: Environment & Scaffolding** - Toolchain setup, Tauri v2 scaffold, folder anatomy.
- [x] **Phase 1: IPC Bridge** - Rust `lookup_word` command, typed definitions, serde serialization, and frontend `invoke`.
- [x] **Phase 2: Offline SQLite Dictionary** - SQLite schema, Python build script, exact match, and morphological lemma fallback in Rust.
- [x] **Phase 3: Global Shortcut & Text Capture** - System-wide hotkey listener (`Ctrl+Alt+D`) and active word capture.
- [x] **Phase 4: Floating Cursor Popup** - Frameless, auto-positioning window at mouse cursor with blur/Esc auto-dismiss.
- [x] **Phase 5: Full Wiktionary SQLite Pipeline** - Batch ingestion script for Kaikki.org English Wiktionary dataset with high-throughput pragmas.
- [x] **Phase 6.1: In-Reader EPUB Canvas & Typography** - Pure-Rust EPUB extraction, Base64 image inlining, Table of Contents, and typography controls.
- [x] **Phase 6.2: Word Bank & Device-First Lookup Frequency** - Local SQLite vocabulary tracking (`user_data.db`), UPSERT counting, badges, and searchable Word Bank drawer.
- [ ] **Phase 7: EPUB Library & Favorites Grid** - Local cover-art library grid, "Read" status, and favorites collection.
- [ ] **Phase 8: Highlights & Annotations** - In-reader text highlighting and saved notes drawer.

---

## License

MIT
