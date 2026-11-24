# LookMeUp

> A distraction-free, desktop-wide background companion providing instantaneous, 100% offline dictionary lookups across any application.

---

## About The Project

LookMeUp runs silently in the background as an operating-system-wide companion. Whenever you encounter an unfamiliar word while reading an article, ebook, paper, or code in any app (Chrome, Sumatra PDF, Kindle, Calibre, Word, VS Code), simply highlight the word and press your global hotkey (default: `Ctrl+Alt+D`).

LookMeUp automatically captures the word, queries its offline B-tree SQLite dictionary, and displays a floating definition card right at your cursor in under a millisecond with zero network requests and zero distractions.

### Key Highlights
* **Universal Desktop Companion:** Works across every application on your machine without requiring plugins or browser extensions.
* **Automatic Word Capture:** Select any text and press your hotkey. LookMeUp automatically copies the highlighted text and resolves the definition without needing manual `Ctrl+C`.
* **Configurable Global Hotkey:** Choose your preferred shortcut chord (e.g. `Ctrl+Alt+D`, `Ctrl+Shift+D`, `Alt+D`, or any custom key combination) directly in Settings.
* **Instant Offline SQLite (<1 ms):** Directly queries a local, pre-indexed SQLite B-tree database with 224,954 headwords and 218,157 morphological lemma fallbacks (e.g. "epiphanies" routes to "epiphany", "running" to "run").
* **Word Bank & Frequency Tracker:** Automatically records your lookups on-device in `user_data.db`. See your device-first lookup frequency (`2x`, `3x`), save words with one click, and browse or search your personal Word Bank anytime.
* **System Tray Resident:** Minimizes cleanly to the Windows notification area / system tray so it is always ready without cluttering your taskbar.
* **Complete Privacy:** 100% offline. Zero network calls, zero telemetry, zero accounts.

---

## Architecture

```
+-------------------------------------------------------------+
|        Any Desktop App (Kindle, Sumatra, Chrome, Word)      |
|                                                             |
|   1. User selects word and presses Global Hotkey            |
+------------------------------+------------------------------+
                               | Win32 Auto-Capture / Hotkey
+------------------------------v------------------------------+
|                     Tauri v2 Rust Core                      |
|                                                             |
|   - Global shortcut hook (tauri-plugin-global-shortcut)     |
|   - Win32 simulated input capture (user32 keybd_event)      |
|   - Instant exact match & lemma fallback query (rusqlite)   |
|   - Cursor coordinate detection & floating window placement |
|   - System Tray management (tauri::tray::TrayIconBuilder)   |
+------------------------------+------------------------------+
                               |
               Tauri IPC (invoke / JSON)
                               |
+------------------------------v------------------------------+
|             Floating Definition Card (Webview)              |
|                                                             |
|   - Frameless, translucent dark-mode card near cursor       |
|   - Sub-millisecond render with vanilla TypeScript & CSS    |
|   - Auto-hides on Esc or blur                               |
|   - Expandable into full Word Bank & Settings manager       |
+-------------------------------------------------------------+
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
|                              | | - Custom hotkey settings   |
+------------------------------+ +----------------------------+
```

---

## Tech Stack

* **Core Runtime:** Tauri v2 (Rust 2024 edition)
* **Backend:** Rust, `rusqlite` (bundled SQLite3), `serde` / `serde_json`, Win32 API (`user32.dll`)
* **Plugins:** `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`, `tauri-plugin-dialog`
* **Frontend:** Vanilla TypeScript, Vite, CSS3 (frameless floating card, glassmorphic Word Bank)
* **Data Pipeline:** Python script compiling 224,954 Webster's Unabridged (1913) and Open English WordNet entries into an indexed, sub-millisecond SQLite database (`dictionary.db`) with 218,157 morphological lemma transformations.

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
- [x] **Phase 6.1: Win32 Auto-Capture on Hotkey** - Synthesizes non-intrusive copy commands on shortcut press to automatically acquire selected text from any foreground desktop app.
- [x] **Phase 6.2: Configurable Global Hotkey** - SQLite settings persistence and dynamic runtime shortcut re-registration via Settings UI and System Tray.
- [x] **Phase 6.3: System Tray Resident & Auto-Dismiss** - Runs silently in the Windows system tray with left-click toggle, right-click context menu, and blur/Esc auto-dismissal.
- [x] **Phase 6.4: Word Bank, Frequency Badges, & Saved Words** - Local vocabulary management in `user_data.db`, UPSERT tracking, "All Lookups" vs. "Saved Words" filter tabs, and frequency badges (`2x`, `3x`).
- [ ] **Phase 7: Offline Text-to-Speech & IPA Phonetics** - In-popup pronunciation guide and native OS speech synthesis audio.
- [ ] **Phase 8: Anki & Flashcard Export** - One-click export of saved words and definitions into Anki decks and CSV format for spaced repetition.
- [ ] **Phase 9: Production Packaging & Windows Installer** - Standalone NSIS / MSI installer with auto-start on boot option.

---

## License

MIT
