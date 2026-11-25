# Learning Log - LookMeUp (Tauri v2 Desktop App)

## Step 0.1: Toolchain & Prerequisites Setup
* **What we did:** Verified Node.js/npm and installed Rust (`rustc`, `cargo`) using `rustup`. Switched the default Rust toolchain to `stable-x86_64-pc-windows-gnu` to leverage the existing MSYS2 GCC 14.2 toolchain as the native linker. Verified linking with a test binary.
* **Key Concept:** Rust compilation is two-stage, just like C++: `rustc` compiles code into object files, but linking into a native Windows `.exe` requires an external linker and C runtime libraries (either MSVC `link.exe` or MinGW/GCC `gcc.exe`).
* **Gotcha:** On Windows, Rust defaults to `x86_64-pc-windows-msvc`. Without the Visual Studio C++ build tools installed, linking fails with `linker link.exe not found`. Using the GNU target (`x86_64-pc-windows-gnu`) allows linking against an existing MinGW/GCC toolchain.

## Step 0.2: Project Scaffolding & Architecture Anatomy
* **What we did:** Scaffolded a Tauri v2 project using `create-tauri-app` configured with vanilla TypeScript, Vite, and npm. Explored the project structure, including frontend (`src/`), Rust backend (`src-tauri/`), `tauri.conf.json`, `Cargo.toml`, and `capabilities/default.json`.
* **Key Concept:** Separation of concerns between the native host and the webview renderer. The frontend is a standard web bundle (bundled by Vite to `dist/`), while `src-tauri/` is an independent Rust package compiling into the native host binary. Tauri v2 introduces a granular capability-based security model (`src-tauri/capabilities/`) where frontend access to native APIs must be explicitly allowed.
* **Gotcha:** In Tauri v2, permissions are not declared globally in `tauri.conf.json` anymore (unlike Tauri v1). Omitting a permission in `capabilities/*.json` results in silent or explicit IPC rejections from the frontend.

## Step 1.1: Rust Data Model, Serde, and IPC Command
* **What we did:** Defined the `Definition` struct in `src-tauri/src/lib.rs`, derived `Serialize` and `Deserialize` using `serde`, implemented the `lookup_word` command returning `Result<Definition, String>`, and registered it in Tauri's `invoke_handler`.
* **Key Concept:** Tauri IPC relies on JSON serialization across the process boundary. `#[derive(Serialize)]` generates compile-time serialization routines without runtime reflection or manual JSON mapping. Returning `Result<T, E>` maps directly to JavaScript Promises (`Ok` -> resolved promise, `Err` -> rejected promise).
* **Gotcha:** In Rust, declaring `pub struct Definition` only makes the struct type public; its fields remain private to the module by default unless each field is explicitly marked `pub`.

## Step 1.2: Frontend IPC Integration with TypeScript
* **What we did:** Defined the matching TypeScript `interface Definition`, replaced the starter greeting with an async `lookup()` function using `invoke<Definition>("lookup_word", { word })`, added try/catch error handling, and wired it up to the submit event listener in `src/main.ts`.
* **Key Concept:** End-to-end typed IPC contract. `invoke` in `@tauri-apps/api/core` sends a message to the native Rust process. Rust deserializes arguments, executes `lookup_word`, serializes the `Definition` result, and resolves the TypeScript Promise. If Rust returns `Err`, the Promise rejects and triggers the JavaScript `catch` block.
* **Gotcha:** Argument names passed in the JavaScript object (`{ word }`) must match the Rust parameter names (`word: String`) exactly; mismatch causes deserialization failure at the IPC layer.



notes:
- learn:
- b-tree indexing in SQLite


## Step 2.1: SQLite Dictionary Schema and Python Database Builder
* **What we did:** Designed an indexed relational schema (`entries` and `forms` tables with `COLLATE NOCASE` indexes) and wrote a Python builder script (`scripts/build_sample_db.py`) to generate a local SQLite database (`dictionary.db`) using prepared statements and transactions.
* **Key Concept:** B-tree indexed offline storage vs. in-memory JSON. An indexed SQLite lookup performs binary search over disk pages in O(log N) time, achieving sub-millisecond retrieval with near-zero idle RAM overhead compared to parsing megabytes of JSON into memory.
* **Gotcha:** When executing SQL queries, parameter placeholders (`?`) must be used instead of string formatting. Parameterized queries compile the SQL AST once and safely bind raw values, preventing syntax crashes on words with quotes (like `o'clock`) and eliminating SQL injection.


## Step 2.2: SQLite Querying, Morphological Fallback, and Latency Measurement in Rust
* **What we did:** Added `rusqlite` with the `bundled` feature, implemented exact-match querying on `entries`, and added morphological fallback on the `forms` table inside `lookup_word` in `src-tauri/src/lib.rs`. Timed query duration using `std::time::Instant`.
* **Key Concept:** Morphological lemma resolution and zero-cost error propagation. Using `rusqlite::OptionalExtension` converts "not found" SQL states into `Ok(None)` instead of hard errors. If exact lookup yields `None`, a secondary index lookup on `forms` resolves inflected forms (`scored` -> `score`) before returning `Ok(Definition)`.
* **Gotcha:** Relative paths in desktop development: during `tauri dev`, the Rust binary's working directory is `src-tauri/`, making `../dictionary.db` resolve to the workspace root. In production distribution, assets must be accessed via Tauri's resource directory API (`app.path().resource_dir()`).


## Step 2.3: Windows PE Export Ordinal Limit & MinGW Linker Resolution
* **What we did:** Diagnosed and resolved the linker error `export ordinal too large: 90667` by removing `"cdylib"` from `crate-type` in `src-tauri/Cargo.toml`.
* **Key Concept:** Windows PE/COFF Export Address Table (EAT) architecture. Windows `.dll` files index exported symbols using 16-bit unsigned integers, capping total exported symbols at 65,535. When building a `cdylib` with deep dependencies (Tauri, WebView2, SQLite, ICU), the number of exported symbols exceeded 65,535, causing GNU `ld.exe` to fail.
* **Gotcha:** Tauri scaffolds `crate-type = ["staticlib", "cdylib", "rlib"]` by default for cross-compiling to mobile (Android requires a `.so` cdylib). For desktop desktop binaries (`main.rs`), `cdylib` is not needed; `["staticlib", "rlib"]` is sufficient and avoids the 16-bit PE export limit under MinGW.


## Step 3.1: Tauri v2 Plugin Architecture & Capability Permissions
* **What we did:** Installed `@tauri-apps/plugin-dialog` (frontend) and `tauri-plugin-dialog` (Rust), registered the plugin in `src-tauri/src/lib.rs`, and configured the permission `dialog:default` in `src-tauri/capabilities/default.json`.
* **Key Concept:** Tauri v2 plugin and security architecture. In v2, native OS features (dialogs, filesystem, notifications) are decentralized into standalone plugins. Even when a plugin is registered in Rust, frontend invocations are blocked by default unless the window's capability manifest explicitly lists the required permissions (principle of least privilege).
* **Gotcha:** Attempting to call `@tauri-apps/plugin-dialog` from JavaScript without adding `"dialog:default"` (or specific sub-permissions like `"dialog:allow-open"`) to `capabilities/default.json` results in a runtime security denial rejection.


## Step 3.0: Architectural Pivot - System-Wide Companion vs. Monolithic Reader
* **What we did:** Evaluated the software engineering tradeoffs between building a custom EPUB reader vs. a universal system-wide desktop companion. Refactored the project roadmap and architecture in `README.md`.
* **Key Concept:** The Unix philosophy: "Do one thing and do it well." Building a full-fledged EPUB reader incurs immense scope creep (pagination, reflow, bookmarks, typography controls, file formats) and forces users to abandon their preferred reading software. Transforming LookMeUp into a system-wide companion allows users to keep their favorite readers while giving them instant offline lookups across any app on the operating system.
* **Gotcha:** Process isolation on modern desktop OSes prevents arbitrary code injection into third-party apps. Clean, OS-sanctioned interaction requires using global shortcuts, clipboard/accessibility APIs, and a floating overlay window rather than in-process DLL injection.


## Step 3.1: OS-Level Global Shortcut Registration in Tauri v2
* **What we did:** Added `tauri-plugin-global-shortcut` to `src-tauri/Cargo.toml`, configured its permission in `src-tauri/capabilities/default.json`, and registered a system-wide hotkey (`Ctrl+Alt+D`) in the `setup` hook in `src-tauri/src/lib.rs`.
* **Key Concept:** Win32 `RegisterHotKey` vs. in-window DOM key listeners. While standard webview listeners only receive events when focused, global shortcuts hook into the OS keyboard dispatcher at the system level. When the key chord is pressed anywhere in Windows, the OS signals the background Rust process directly.
* **Gotcha:** Global shortcuts are system-wide unique registrations per session. If another active application has already claimed the exact key combination (or if unregister is not handled on shutdown), registration can fail. Handling errors gracefully in the `.setup()` hook is essential.


## Step 3.2: Clipboard Text Capture on Global Shortcut
* **What we did:** Added `tauri-plugin-clipboard-manager` to `Cargo.toml`, permitted clipboard read/write in `capabilities/default.json`, and hooked clipboard text extraction into the `Ctrl+Alt+D` global shortcut listener in `src-tauri/src/lib.rs`.
* **Key Concept:** Cross-process text acquisition. Operating system security boundaries prevent directly querying the DOM or text selection of third-party apps. Using the system clipboard provides a universal, permission-safe communication channel to retrieve text selected by the user in any reader, browser, or editor.
* **Gotcha:** Syntax placement in Rust: method chains (`.plugin(...)`) and executable statements cannot exist at global module scope. They must live inside a function body (such as `pub fn run()`), chained to the builder instance.


## Step 4.1: Frameless Floating Popup Window & Cursor Positioning
* **What we did:** Configured a frameless, transparent, floating (`alwaysOnTop`), `skipTaskbar` window in `src-tauri/tauri.conf.json`. In `src-tauri/src/lib.rs`, queried mouse cursor coordinates via `app.cursor_position()` on global shortcut press, repositioned the popup next to the active cursor, and emitted `"show-definition"` event payloads.
* **Key Concept:** Desktop overlay architecture. Setting `decorations: false` strips OS chrome (titlebar and min/max/close controls), while `transparent: true` delegates shape clipping to CSS border-radius and background blur. Combining `alwaysOnTop: true` with physical coordinate offsets positions the companion directly in the user's line of sight without taking over window management.
* **Gotcha:** Rust's `app.cursor_position()` returns physical monitor pixels (`PhysicalPosition`), while webviews often think in logical pixels (DPI scaling). Passing `Position::Physical` directly ensures pixel-perfect window placement regardless of Windows display scale (125%, 150%, etc.).

## Step 4.2: Floating Card UI, Event Listening, and Auto-Dismiss
* **What we did:** Redesigned the frontend into a glassmorphic floating dark-mode card, implemented the `listen("show-definition")` event subscriber in TypeScript, and added native auto-dismiss handlers via `invoke("hide_window")` on `Escape` keypress and window `blur`.
* **Key Concept:** Ephemeral UX and event-driven rendering. Unlike conventional windowed applications that require manual closing, reading companions must be ephemeral. Subscribing to window `blur` (loss of focus) and `Escape` ensures the card vanishes the instant the user continues reading, maintaining undisturbed focus.


## Step 5.1: High-Throughput SQLite Bulk Ingestion & Post-Indexing
* **What we did:** Created `scripts/build_full_db.py` to stream Wiktionary JSON Lines data (including `.gz` compressed files) into SQLite. Configured bulk ingestion pragmas (`PRAGMA synchronous = OFF`, `PRAGMA journal_mode = MEMORY`, `PRAGMA cache_size = 100000`), batched inserts, and deferred index creation until after all rows were populated.
* **Key Concept:** Write amplification and batch transactions in relational databases. Default SQLite autocommit triggers an OS `fsync` per row, throttling inserts to ~100 rows/second. Disabling synchronous disk flushes during compilation and batching inserts across explicit transactions accelerates throughput to 100,000+ rows/second. Deferring B-tree index creation transforms $N$ separate index rebalancing operations into a single $O(N \log N)$ bulk sort.
* **Gotcha:** Disabling `synchronous` and using `journal_mode = MEMORY` is safe during build/ETL scripts (since the entire database can simply be rebuilt if the process crashes), but should never be used for user data where power loss could cause corruption.


## Step 5.2: Touch-Friendly In-Reader Canvas & Single-Popup Selection
* **What we did:** Reset `tauri.conf.json` to a standard full desktop reader window (`1024x720`, decorated, visible). Implemented an in-reader canvas with comfortable typography, integrated word selection capture via `pointerup` and `window.getSelection()`, and positioned an inline floating dictionary card directly above the tapped word using `range.getBoundingClientRect()`.
* **Key Concept:** Direct-manipulation touch UX vs. global shortcut collision. When reading in external apps, competing selection bubbles overlap and touchscreens lack physical keyboards. Hosting the reading canvas directly gives LookMeUp 100% control over the DOM: zero competing popups, touch/tap-friendly selection without hotkeys, and pixel-perfect relative positioning above the selected word.
* **Gotcha:** Windows file locking (`os error 32: ERROR_SHARING_VIOLATION`). On Windows, running binaries cannot be overwritten by the linker. If `tauri dev` or `app.exe` is running when Cargo attempts to link, the build fails with a sharing violation until the running process is stopped.


## Step 5.3: Window Label Binding & Dialog Permission Resolution
* **What we did:** Added explicit `"label": "main"` to the reader window in `src-tauri/tauri.conf.json` and wrapped the frontend file picker in `try/catch` with user-facing error reporting in `src/main.ts`.
* **Key Concept:** Capability targeting in Tauri v2. In `capabilities/default.json`, permissions (such as `"dialog:default"`) are bound to specific window labels (`"windows": ["main"]`). If a window definition in `tauri.conf.json` omits the `"label"` property, capability binding fails to match the window, causing API calls like `open()` to be rejected silently at runtime.
* **Gotcha:** Always ensure the window `label` in `tauri.conf.json` matches the strings in `capabilities/*.json` windows array, and always wrap asynchronous plugin promises in `try/catch` blocks in TypeScript to prevent unhandled rejection silence.


## Step 5.4: Native Rust EPUB Extraction & Direct-DOM Rendering
* **What we did:** Added the pure-Rust `zip` crate to `src-tauri/Cargo.toml`, implemented `load_epub` in `src-tauri/src/lib.rs` to extract and parse readable XHTML/HTML chapters from `.epub` archives, and wired chapter rendering and pagination controls into `src/main.ts` and `index.html`.
* **Key Concept:** Direct-DOM rendering vs. nested iframe architectures. Rather than embedding complex iframe-based web viewers (`epub.js`) which suffer from CORS restrictions and distorted touch selection coordinates, unpacking EPUB chapters natively in Rust and rendering into the root DOM enables zero-overhead text selection (`window.getSelection()`), instant word lookup, and fluid touch interactions across the entire book.
* **Gotcha:** EPUBs are ZIP containers containing diverse internal structures (some prefix with `OEBPS/`, others use root directories). Extracting readable body markup while filtering out navigation tables (`nav.xhtml`, `toc.ncx`) ensures clean, readable prose without formatting clutter.


## Step 6.1: Base64 EPUB Image Inlining & Typography Customizer
* **What we did:** Implemented in-memory image extraction and Base64 Data URI replacement in `src-tauri/src/lib.rs` for EPUB chapters, and built the full typography customization popover (font family, font size, line spacing, word spacing, paragraph spacing, 4-button text alignment, and reset button) matching the user interface design. Added persistent settings via `localStorage` and a slide-out Table of Contents chapter drawer.
* **Key Concept:** Self-contained Data URIs vs. relative asset references in webview sandbox. Relative paths (`../images/pic.jpg`) inside unpacked HTML fail because webviews resolve them against the host server origin (`localhost:1420`). Converting image bytes to `data:image/...;base64,...` inlines assets directly into the HTML document, guaranteeing instant, self-contained rendering with zero webview security or CORS hurdles.
* **Gotcha:** EPUB file structures vary widely across publishers. Image files can be located in root or subdirectories (`OEBPS/images/`, `images/`, etc.). Indexing images by both full path and basename ensures that varying relative path references (`src="images/..."`, `src="../images/..."`) match and resolve properly.

## Step 6.2: Word Bank & Device-First Lookup Frequency Tracking
* **What we did:** Created a dedicated `user_data.db` SQLite database with a `word_history` table. Implemented atomic UPSERT tracking inside `lookup_word` in Rust to increment device-first frequency counts and timestamps whenever a word is inspected. Added the Word Bank slide-out drawer with real-time search filtering, delete actions, and frequency badges (`2x`, `3x`) on both the active reading popup and the vocabulary cards.
* **Key Concept:** Data partitioning (read-only reference DB vs. mutable user data DB) and SQLite atomic UPSERT. Keeping `dictionary.db` strictly read-only allows clean updates and distribution, while mutable user data (lookups, counts, bookmarks) lives in `user_data.db`. Using `INSERT ... ON CONFLICT(word) DO UPDATE SET lookup_count = lookup_count + 1` handles both first-time discoveries and repeat encounters in a single atomic database query.
* **Gotcha:** Storing timestamps as 64-bit integer Unix seconds (`i64`) prevents timezone skew and locale formatting bugs in SQLite, ensuring fast indexed sorting (`ORDER BY lookup_count DESC, last_looked_up DESC`) without runtime date parsing overhead.

## Step 6.3: Explicit "Save Word" Option, Schema Migration, & Saved Vocabulary Tabs
* **What we did:** Added a prominent "Save Word" button with bookmark icon directly in the reading definition popup header, added the `is_saved` column to `word_history` in `user_data.db`, implemented `toggle_save_word` in Rust, and created "All Lookups" and "Saved Words" tab filters with live count badges in the Word Bank drawer. Also allowed pressing Enter in the search bar to look up and save any word immediately.
* **Key Concept:** Idempotent schema migrations in SQLite (`ALTER TABLE ... ADD COLUMN ...`). When evolving an existing SQLite database without dropping user data, attempting to run `ALTER TABLE ADD COLUMN` safely ensures new columns are present while gracefully ignoring errors if the column already exists.
* **Gotcha:** Multi-view state synchronization: saving a word from the floating popup must immediately update the corresponding card in the slide-out drawer and recompute the tab counters (`count-all`, `count-saved`) to avoid stale UI state.

## Step 6.4: Full 225,000+ Word Dictionary Compilation & Morphological Ingestion
* **What we did:** Implemented `scripts/build_full_db.py` to compile 388,587 raw lexical records from Webster's Revised Unabridged Dictionary (1913) and Open English WordNet into `dictionary.db`. Aggregated 224,954 unique headwords, generated 218,157 irregular and rule-based morphological forms (`forms` table), and constructed `COLLATE NOCASE` B-tree indexes. Achieved sub-millisecond retrieval (0.16 ms to 0.59 ms) for both exact and inflected lookups.
* **Key Concept:** Pre-compiled relational morphology vs. runtime NLP lemmatization. Heavy runtime NLP libraries (like spaCy or NLTK) add tens of megabytes of bloat, slow startup times, and complex runtime dependencies. Pre-computing morphological transformations into an indexed SQLite table (`forms`) offloads linguistic computation entirely to compile-time, reducing runtime lookups to a single lightning-fast O(log N) B-tree page read.
* **Gotcha:** Atomic database replacement. Writing hundreds of thousands of rows into an active database file risks lock contention and partial-state crashes. Building the entire dataset and its indexes into a temporary database (`dictionary.db.tmp`) before performing an atomic OS rename guarantees that running application processes never observe an incomplete or corrupt dictionary state.

## Step 7.1: Win32 Auto-Copy Simulation & Rust 2024 Unsafe FFI
* **What we did:** Re-architected LookMeUp from an in-app reader into a desktop-wide background companion. Implemented Win32 input simulation via `user32::keybd_event` to automatically trigger `Ctrl+C` in whatever application the user is currently reading (e.g. Chrome, Sumatra PDF, Kindle, Word) whenever the global hotkey is pressed.
* **Key Concept:** Foreign Function Interface (FFI) and `unsafe` boundaries in Rust 2024. In the 2024 edition, foreign extern declarations must be explicitly marked `unsafe extern "system"` and calling unsafe functions requires an explicit `unsafe` block even inside an `unsafe fn` (`unsafe_op_in_unsafe_fn`). Wrapping raw Win32 system calls in safe Rust abstractions encapsulates platform-specific invariants cleanly.
* **Gotcha:** Keyboard event race conditions in OS event dispatchers. When an application registers a global shortcut chord (e.g. `Ctrl+Alt+D`), the user may still have physical modifier keys depressed when the hotkey event fires. Inserting a slight timing gap (40-70 ms) ensures the OS has acknowledged modifier key release before synthesizing the `Ctrl+C` copy sequence.

## Step 7.2: Configurable Global Shortcuts & System Tray Architecture in Tauri v2
* **What we did:** Added `app_settings` to `user_data.db` to persist user hotkey preferences across restarts. Built `get_current_hotkey` and `set_current_hotkey` commands in Rust to dynamically unregister the existing shortcut and bind the new key combination at runtime. Implemented a native System Tray icon with `tauri::tray::TrayIconBuilder` featuring quick lookup, Word Bank navigation, and graceful application exit.
* **Key Concept:** Dynamic resource rebinding without process restart. In desktop operating systems, global shortcuts are held in OS-level registration tables. Changing a shortcut requires unregistering the old chord from the system table, parsing the new syntax, and atomically registering the updated listener callback.
* **Gotcha:** Window visibility and taskbar behavior for companion apps. Setting `skipTaskbar: true` and `alwaysOnTop: true` ensures the companion card appears immediately over the target text without polluting the Windows Alt-Tab or Taskbar queues, while auto-dismissing on `blur` or `Escape` preserves undisturbed reading flow.

## Step 7.3: Offline IPA Phonetics & Native Windows SAPI Text-to-Speech
* **What we did:** Integrated 125,927 US English IPA phonetic transcriptions (`ipa-dict` `en_US.txt`) into `dictionary.db` via `scripts/build_full_db.py`, added an idempotent `ipa` column migration to `user_data.db`, implemented a `speak_word` Rust command backed by Windows `System.Speech.Synthesis.SpeechSynthesizer`, and added the IPA badge and speaker button to the floating card and Word Bank.
* **Key Concept:** Hybrid offline TTS with zero-bloat fallback and safe process invocation. Using the browser's `window.speechSynthesis` when voices are loaded provides 0 ms playback latency, while spawning Windows SAPI in a background thread with `CREATE_NO_WINDOW` (`0x08000000`) guarantees offline pronunciation even if WebView2 voices are unavailable. Passing the word via an environment variable (`LOOKMEUP_SPEAK_WORD`) rather than string-interpolating into the PowerShell command eliminates shell quoting bugs and injection vectors.
* **Gotcha:** Database schema evolution across both read-only and mutable databases. When adding a new column (`ipa`) to `dictionary.db`, rebuilding the read-only artifact is sufficient, whereas `user_data.db` holds persistent user history and requires an idempotent `ALTER TABLE word_history ADD COLUMN ipa TEXT NOT NULL DEFAULT ''` migration on startup.

## Step 7.4: One-Click Anki & RFC-4180 CSV Flashcard Export
* **What we did:** Added `export_word_bank` in `src-tauri/src/lib.rs` to export either Saved Words or All Lookups into Anki-ready tab-separated decks (`.txt` with `#separator:tab`, `#html:true`, `#tags column:3`) or RFC-4180 escaped `.csv` spreadsheets. Added **Export Anki** and **Export CSV** buttons in the Word Bank toolbar wired to the native OS Save dialog via `@tauri-apps/plugin-dialog`.
* **Key Concept:** Structured plain-text interchange formats without heavy binary dependencies. Anki natively imports tab-separated text files with metadata directives (`#separator:tab`, `#html:true`), allowing rich HTML formatting (`<b>`, `<ol>`, IPA, frequency badges, and tags) without bundling SQLite/ZIP `.apkg` generators. Similarly, RFC-4180 CSV escaping (doubling internal quotes and quoting fields with commas/newlines) ensures definitions containing punctuation open cleanly in Excel and Google Sheets.
* **Gotcha:** Tab and newline sanitization in TSV exports. Because Anki treats raw tabs (`\t`) and newlines (`\n`) as column and card delimiters, any stray control characters inside dictionary glosses must be normalized to spaces and HTML-escaped before writing each row.

## Step 7.5: Production Resource Paths, First-Run Onboarding, & NSIS Installer
* **What we did:** Replaced hardcoded relative dev paths (`../dictionary.db`, `../user_data.db`) with `OnceLock<PathBuf>` resolvers backed by Tauri's `resource_dir()` (checking both `dictionary.db` and `_up_/dictionary.db`) and `app_data_dir()`. Added a race-free `check_first_run` command that opens the Settings view on first launch so new users can choose their hotkey immediately. Configured `"targets": ["nsis"]` and `"resources": ["../dictionary.db"]` in `src-tauri/tauri.conf.json` and built the standalone Windows installer (`LookMeUp_0.1.0_x64-setup.exe`, **16.22 MiB**).
* **Key Concept:** Read-only bundle resources vs. writable OS user data directories. Installed Windows applications live under `C:\Program Files\`, which is read-only for standard users. Bundling `dictionary.db` as a Tauri resource while placing `user_data.db` in `%APPDATA%` (`app_data_dir()`) respects OS security boundaries and avoids UAC permission errors.
* **Gotcha:** When `tauri.conf.json` bundles a parent-directory resource like `"../dictionary.db"`, Tauri places it inside `_up_/dictionary.db` under `resource_dir()`. Checking both `resource_dir().join("dictionary.db")` and `resource_dir().join("_up_").join("dictionary.db")` ensures reliable lookup across all bundle layouts.
