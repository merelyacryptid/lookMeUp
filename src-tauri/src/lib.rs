// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::time::Instant;
use tauri::{Emitter, Manager, PhysicalPosition, Position};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubChapter {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubBook {
    pub title: String,
    pub cover_data: Option<String>,
    pub chapters: Vec<EpubChapter>,
}

#[tauri::command]
fn load_epub(path: String) -> Result<EpubBook, String> {
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read EPUB zip archive: {}", e))?;

    let book_title = std::path::Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled Book")
        .to_string();

    // 1. Extract all images and convert to Base64 Data URIs
    let mut image_map: HashMap<String, String> = HashMap::new();
    let mut cover_data: Option<String> = None;

    for i in 0..archive.len() {
        if let Ok(mut entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            let lower = name.to_lowercase();
            let mime = if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
                Some("image/jpeg")
            } else if lower.ends_with(".png") {
                Some("image/png")
            } else if lower.ends_with(".gif") {
                Some("image/gif")
            } else if lower.ends_with(".webp") {
                Some("image/webp")
            } else if lower.ends_with(".svg") {
                Some("image/svg+xml")
            } else {
                None
            };

            if let Some(mime_type) = mime {
                let mut bytes = Vec::new();
                if entry.read_to_end(&mut bytes).is_ok() {
                    let b64 = BASE64_STANDARD.encode(&bytes);
                    let data_uri = format!("data:{};base64,{}", mime_type, b64);

                    // Check for cover image candidate
                    if cover_data.is_none() && (lower.contains("cover") || lower.contains("titlepage")) {
                        cover_data = Some(data_uri.clone());
                    }

                    // Map by both full archive path and basename
                    let basename = name.split('/').last().unwrap_or(&name).to_string();
                    image_map.insert(name.clone(), data_uri.clone());
                    image_map.insert(basename, data_uri);
                }
            }
        }
    }

    // 2. Discover chapter files
    let mut chapter_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            let lower = name.to_lowercase();
            if (lower.ends_with(".xhtml") || lower.ends_with(".html") || lower.ends_with(".htm"))
                && !lower.contains("nav.") && !lower.contains("toc.")
            {
                chapter_names.push(name);
            }
        }
    }

    chapter_names.sort();

    // 3. Extract and inline images into chapter bodies
    let mut chapters = Vec::new();
    for (idx, name) in chapter_names.iter().enumerate() {
        if let Ok(mut entry) = archive.by_name(name) {
            let mut raw_html = String::new();
            if entry.read_to_string(&mut raw_html).is_ok() {
                // Extract <body> content if present
                let mut body_content = if let Some(start) = raw_html.find("<body") {
                    if let Some(inner_start) = raw_html[start..].find('>') {
                        let after_body = &raw_html[start + inner_start + 1..];
                        if let Some(end) = after_body.rfind("</body>") {
                            after_body[..end].to_string()
                        } else {
                            after_body.to_string()
                        }
                    } else {
                        raw_html
                    }
                } else {
                    raw_html
                };

                // Replace image references with embedded Base64 data URIs
                for (img_key, data_uri) in &image_map {
                    // Match variations like src="images/pic.jpg", src="../images/pic.jpg", xlink:href="..."
                    let patterns = [
                        format!("src=\"{}\"", img_key),
                        format!("src='{}'", img_key),
                        format!("href=\"{}\"", img_key),
                        format!("href='{}'", img_key),
                    ];
                    for pat in &patterns {
                        if body_content.contains(pat) {
                            let replacement = format!("src=\"{}\"", data_uri);
                            body_content = body_content.replace(pat, &replacement);
                        }
                    }
                }

                chapters.push(EpubChapter {
                    title: format!("Chapter {}", idx + 1),
                    content: body_content,
                });
            }
        }
    }

    if chapters.is_empty() {
        return Err("No readable chapters found in this EPUB file.".to_string());
    }

    Ok(EpubBook {
        title: book_title,
        cover_data,
        chapters,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub word: String,
    pub part_of_speech: String,
    pub meanings: Vec<String>,
    pub lookup_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordHistoryItem {
    pub word: String,
    pub part_of_speech: String,
    pub meanings: Vec<String>,
    pub lookup_count: u32,
    pub last_looked_up: i64,
}

fn get_user_db() -> Result<Connection, String> {
    let conn = Connection::open("../user_data.db").map_err(|e| format!("Failed to open user DB: {}", e))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS word_history (
            word TEXT PRIMARY KEY COLLATE NOCASE,
            part_of_speech TEXT NOT NULL,
            definition_json TEXT NOT NULL,
            lookup_count INTEGER NOT NULL DEFAULT 1,
            first_looked_up INTEGER NOT NULL,
            last_looked_up INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_word_history_count ON word_history(lookup_count DESC);
        CREATE INDEX IF NOT EXISTS idx_word_history_last ON word_history(last_looked_up DESC);"
    ).map_err(|e| format!("Failed to initialize word_history table: {}", e))?;
    Ok(conn)
}

fn record_lookup(def: &Definition) -> u32 {
    if let Ok(conn) = get_user_db() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let json_meanings = serde_json::to_string(&def.meanings).unwrap_or_default();

        let _ = conn.execute(
            "INSERT INTO word_history (word, part_of_speech, definition_json, lookup_count, first_looked_up, last_looked_up)
             VALUES (?1, ?2, ?3, 1, ?4, ?4)
             ON CONFLICT(word) DO UPDATE SET
                 lookup_count = lookup_count + 1,
                 last_looked_up = ?4;",
            params![def.word, def.part_of_speech, json_meanings, now],
        );

        // Retrieve current count
        conn.query_row(
            "SELECT lookup_count FROM word_history WHERE word = ?1 COLLATE NOCASE",
            params![def.word],
            |row| row.get(0),
        ).unwrap_or(1)
    } else {
        1
    }
}

#[tauri::command]
fn get_word_history() -> Result<Vec<WordHistoryItem>, String> {
    let conn = get_user_db()?;
    let mut stmt = conn
        .prepare("SELECT word, part_of_speech, definition_json, lookup_count, last_looked_up FROM word_history ORDER BY lookup_count DESC, last_looked_up DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let word: String = row.get(0)?;
            let pos: String = row.get(1)?;
            let json_str: String = row.get(2)?;
            let count: u32 = row.get(3)?;
            let last: i64 = row.get(4)?;
            let meanings: Vec<String> = serde_json::from_str(&json_str).unwrap_or_default();
            Ok(WordHistoryItem {
                word,
                part_of_speech: pos,
                meanings,
                lookup_count: count,
                last_looked_up: last,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for r in rows {
        if let Ok(item) = r {
            result.push(item);
        }
    }
    Ok(result)
}

#[tauri::command]
fn delete_history_word(word: String) -> Result<(), String> {
    let conn = get_user_db()?;
    conn.execute("DELETE FROM word_history WHERE word = ?1 COLLATE NOCASE", params![word])
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
fn lookup_word(word: String) -> Result<Definition, String> {
    let start = Instant::now();
    let clean_word = word.trim();
    if clean_word.is_empty() {
        return Err("Word cannot be empty".to_string());
    }

    let conn =
        Connection::open("../dictionary.db").map_err(|e| format!("Failed to open DB: {}", e))?;

    // Helper to fetch an entry by headword
    let fetch_entry = |target: &str| -> Result<Option<Definition>, String> {
        conn.query_row(
            "SELECT word, part_of_speech, meanings FROM entries WHERE word = ?1 COLLATE NOCASE",
            params![target],
            |row| {
                let w: String = row.get(0)?;
                let pos: String = row.get(1)?;
                let m_json: String = row.get(2)?;
                let meanings: Vec<String> = serde_json::from_str(&m_json).unwrap_or_default();
                Ok(Definition {
                    word: w,
                    part_of_speech: pos,
                    meanings,
                    lookup_count: 1,
                })
            },
        )
        .optional()
        .map_err(|e| format!("Query error: {}", e))
    };

    // 1. Try exact match
    let mut result = fetch_entry(clean_word)?;

    // 2. Lemma fallback: check forms table
    if result.is_none() {
        let headword: Option<String> = conn
            .query_row(
                "SELECT headword FROM forms WHERE form = ?1 COLLATE NOCASE",
                params![clean_word],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Query error: {}", e))?;

        if let Some(hw) = headword {
            result = fetch_entry(&hw)?;
        }
    }

    if let Some(ref mut def) = result {
        let count = record_lookup(def);
        def.lookup_count = count;
    }

    println!("Lookup for '{}' took {:?}", clean_word, start.elapsed());

    result.ok_or_else(|| format!("'{}' not found in dictionary", clean_word))
}

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let shortcut: Shortcut = "ctrl+alt+d".parse().map_err(|e| format!("{:?}", e))?;
            app.global_shortcut()
                .on_shortcut(shortcut, |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            // 1. Position near mouse cursor
                            if let Ok(pos) = app.cursor_position() {
                                let _ = window.set_position(Position::Physical(PhysicalPosition {
                                    x: pos.x as i32 + 12,
                                    y: pos.y as i32 + 12,
                                }));
                            }

                            // 2. Read clipboard and query SQLite
                            if let Ok(text) = app.clipboard().read_text() {
                                let trimmed = text.trim();
                                let payload = lookup_word(trimmed.to_string());
                                let _ = window.emit("show-definition", payload);
                            }

                            // 3. Reveal and focus popup
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lookup_word,
            hide_window,
            load_epub,
            get_word_history,
            delete_history_word
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
