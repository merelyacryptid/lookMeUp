// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let shortcut: Shortcut = "ctrl+alt+d".parse().map_err(|e| format!("{:?}", e))?;
            app.global_shortcut().on_shortcut(shortcut, |_app, shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    println!("Global shortcut triggered: {:?}", shortcut);
                }
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![lookup_word])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
