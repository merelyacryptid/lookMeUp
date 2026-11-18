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
