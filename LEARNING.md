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
