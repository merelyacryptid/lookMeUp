# LookMeUp

> A fast, distraction-free desktop reader with instantaneous, 100% offline dictionary lookups for EPUBs and PDFs.

---

## About The Project

As someone who reads a lot, I hated having my reading flow broken every time I hit an unfamiliar word while disconnected from the internet—which happens constantly around my college campus. Most dictionary tools either fail completely without a network connection, leak what you read to third-party APIs, or run on bloated Electron runtimes that eat hundreds of megabytes of RAM just to render text.

LookMeUp was built to fix this with a systems-first mindset:
* **Zero Latency:** Queries a local, pre-indexed SQLite dictionary in under 50 ms directly from Rust.
* **Morphological Fallback:** Automatically resolves inflected forms and conjugations back to their lemma/root form (e.g., selecting "running" or "scored" finds "run" or "score").
* **Lightweight Footprint:** Built on Tauri v2 and vanilla TypeScript, utilizing the operating system's native webview engine instead of bundling Chromium. The entire application binary stays lean (~15 MB) and keeps memory usage minimal.
* **Complete Privacy:** 100% offline. Zero network calls, zero telemetry, zero accounts.

---

## Architecture

LookMeUp operates on an asynchronous native-core architecture:

```
+--------------------------------------------------------+
|               Frontend (OS Native Webview)             |
|                                                        |
|   - Foliate-js (EPUB reader) / PDF.js (PDF reader)     |
|   - Word selection capture and popup UI                |
|   - Vanilla TypeScript + Vite                          |
+---------------------------+----------------------------+
                            |
               Tauri IPC (invoke / JSON)
                            |
+---------------------------v----------------------------+
|                  Backend (Rust Core)                   |
|                                                        |
|   - lookup_word command handler                        |
|   - Fast exact & lemma fallback queries (rusqlite)     |
|   - Strict security and capability sandbox             |
+---------------------------+----------------------------+
                            |
+---------------------------v----------------------------+
|             Local Indexed SQLite Database              |
|                                                        |
|   - Extracted from Kaikki.org (Wiktionary dumps)       |
|   - Headword indexes + form_of lemma mapping table     |
+--------------------------------------------------------+
```

---

## Tech Stack

* **Core Runtime:** Tauri v2 (Rust 2024 edition)
* **Backend:** Rust, `rusqlite` (bundled SQLite3), `serde` / `serde_json`
* **Frontend:** Vanilla TypeScript, Vite, CSS3 (zero UI framework overhead)
* **Reader Engines:** `foliate-js` (EPUBs), `pdf.js` (PDFs)
* **Data Pipeline:** Python script converting Kaikki.org Wiktionary JSON dumps into optimized, indexed SQLite databases

---

## Getting Started

### Prerequisites

* **Node.js:** v20+ with `npm`
* **Rust:** Rust 1.80+ (`rustup`)
* **C++ / Linker Toolchain:**
  * **Windows:** MSYS2 GCC/MinGW (`x86_64-pc-windows-gnu`) or Visual Studio C++ Build Tools (`x86_64-pc-windows-msvc`) + WebView2 (pre-installed on Windows 10/11)
  * **macOS:** Xcode Command Line Tools
  * **Linux:** `webkit2gtk-4.1`, `build-essential`

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/lookmeup.git
   cd lookmeup
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   ```

3. **Start the development server with Tauri desktop window:**
   ```bash
   npm run tauri dev
   ```

4. **Build a production standalone binary:**
   ```bash
   npm run tauri build
   ```

---

## Roadmap

- [x] **Phase 0: Environment & Scaffolding** - Toolchain setup, Tauri v2 scaffold, folder anatomy walkthrough.
- [ ] **Phase 1: IPC Bridge** - Rust `lookup_word` command, typed definitions, serde serialization, and frontend `invoke`.
- [ ] **Phase 2: Offline SQLite Dictionary** - Python build script for Kaikki.org Wiktionary dumps, exact match, and lemma inflection fallback.
- [ ] **Phase 3: EPUB Reader** - File picker dialog plugin, `foliate-js` integration, selection capture.
- [ ] **Phase 4: Definition Popup** - Non-intrusive floating popup, dark mode theme, keyboard shortcuts.
- [ ] **Phase 5: PDF Support** - `pdf.js` integration and text-layer selection.
- [ ] **Phase 6: Polish & Distribution** - History/saved words, packaging & installer distribution.

---

## License

MIT
