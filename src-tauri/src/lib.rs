// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub word: String,
    pub part_of_speech: String,
    pub meanings: Vec<String>,
}
#[tauri::command]
fn lookup_word(word: String) -> Result<Definition, String> {
    if word.trim().is_empty() {
        return Err("Word cannot be empty".to_string());
    }

    Ok(Definition {
        word: word.clone(),
        part_of_speech: "noun".to_string(),
        meanings: vec![
            format!("Mock definition 1 for '{}'", word),
            format!("Mock definition 2 for '{}'", word),
        ],
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![lookup_word])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
