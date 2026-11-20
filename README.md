# LookMeUp

> A lightning-fast, 100% offline system-wide dictionary companion. Look up any word in any EPUB reader, PDF viewer, browser, or desktop app with a single keystroke in under 50 ms.

---

## About The Project

As someone who reads a lot across various apps (EPUB readers, PDF viewers, research papers in browsers), I hated having my reading flow broken every time I ran into an unfamiliar word without an active internet connection (which happens constantly around my college campus). Most online dictionary tools fail without Wi-Fi, leak reading activity to remote APIs, or require copying text into a separate heavy browser tab.

Meanwhile, trying to build a custom EPUB reader from scratch meant reinventing pagination, fonts, themes, bookmarks, and layout engines—all while forcing users to abandon the reading apps they already love (like Kindle, Calibre, SumatraPDF, or Thorium).

**LookMeUp** takes a Unix-philosophy approach: **do one thing with extreme speed and precision.** 

Instead of replacing your reader, LookMeUp runs silently in the background as a universal companion:
* **Works Anywhere:** Highlight a word in *any* reader, PDF viewer, browser, or text editor, press a hotkey (e.g. `Ctrl+Alt+D`), and an instant definition appears.
* **Zero Latency (<50 ms):** Directly queries a local, pre-indexed SQLite B-tree database compiled from Wiktionary dumps.
* **Morphological Fallback:** Automatically resolves inflected verbs and plurals back to their base lemmas (e.g., selecting "running" or "scored" routes to "run" or "score").
* **Floating Minimalist Card:** A sleek, frameless floating popup appears right at your mouse cursor and disappears the moment you hit `Esc` or click away.
* **100% Offline & Private:** Zero network requests, zero telemetry, zero accounts.

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
- [ ] **Phase 3: Global Shortcut & Text Capture** - System-wide hotkey listener (`Ctrl+Alt+D`) and active word capture.
- [ ] **Phase 4: Floating Cursor Popup** - Frameless, auto-positioning window at mouse cursor with blur/Esc auto-dismiss.
- [ ] **Phase 5: Full Wiktionary SQLite Pipeline** - Batch ingestion script for full Kaikki.org English Wiktionary dataset.
- [ ] **Phase 6: System Tray & Polish** - Background tray icon, saved words history, and production installer.

---

## License

MIT
