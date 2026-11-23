// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
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

    let mut chapters = Vec::new();
    for (idx, name) in chapter_names.iter().enumerate() {
        if let Ok(mut entry) = archive.by_name(name) {
            let mut raw_html = String::new();
            if entry.read_to_string(&mut raw_html).is_ok() {
                let body_content = if let Some(start) = raw_html.find("<body") {
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
        chapters,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub word: String,
    pub part_of_speech: String,
    pub meanings: Vec<String>,
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
        .invoke_handler(tauri::generate_handler![lookup_word, hide_window, load_epub])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
