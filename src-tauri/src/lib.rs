// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, LogicalSize, Manager, PhysicalPosition, Position, Size};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg(target_os = "windows")]
fn simulate_ctrl_c() -> bool {
    unsafe extern "system" {
        fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
        fn GetClipboardSequenceNumber() -> u32;
    }
    const VK_SHIFT: u8 = 0x10;
    const VK_CONTROL: u8 = 0x11;
    const VK_MENU: u8 = 0x12; // Alt
    const VK_LWIN: u8 = 0x5B;
    const VK_RWIN: u8 = 0x5C;
    const VK_LSHIFT: u8 = 0xA0;
    const VK_RSHIFT: u8 = 0xA1;
    const VK_LCONTROL: u8 = 0xA2;
    const VK_RCONTROL: u8 = 0xA3;
    const VK_LMENU: u8 = 0xA4;
    const VK_RMENU: u8 = 0xA5;
    const VK_C: u8 = 0x43;
    const KEYEVENTF_KEYUP: u32 = 0x0002;

    let seq_before = unsafe { GetClipboardSequenceNumber() };

    unsafe {
        // Explicitly release any modifier keys held by the user's hotkey chord
        // so the foreground app sees a clean Ctrl+C instead of Ctrl+Alt+C or Ctrl+Shift+C.
        for vk in [
            VK_MENU,
            VK_LMENU,
            VK_RMENU,
            VK_SHIFT,
            VK_LSHIFT,
            VK_RSHIFT,
            VK_LWIN,
            VK_RWIN,
            VK_CONTROL,
            VK_LCONTROL,
            VK_RCONTROL,
        ] {
            keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(25));

    unsafe {
        // Send clean Ctrl + C
        keybd_event(VK_CONTROL, 0, 0, 0);
        keybd_event(VK_C, 0, 0, 0);
        keybd_event(VK_C, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYUP, 0);
    }

    // Poll until the foreground app updates the clipboard (up to 180ms)
    for _ in 0..12 {
        std::thread::sleep(std::time::Duration::from_millis(15));
        let seq_now = unsafe { GetClipboardSequenceNumber() };
        if seq_now != seq_before {
            return true;
        }
    }

    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub word: String,
    pub part_of_speech: String,
    pub ipa: String,
    pub meanings: Vec<String>,
    pub lookup_count: u32,
    pub is_saved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordHistoryItem {
    pub word: String,
    pub part_of_speech: String,
    pub ipa: String,
    pub meanings: Vec<String>,
    pub lookup_count: u32,
    pub is_saved: bool,
    pub last_looked_up: i64,
}

fn get_user_db() -> Result<Connection, String> {
    let conn = Connection::open("../user_data.db")
        .map_err(|e| format!("Failed to open user DB: {}", e))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS word_history (
            word TEXT PRIMARY KEY COLLATE NOCASE,
            part_of_speech TEXT NOT NULL,
            ipa TEXT NOT NULL DEFAULT '',
            definition_json TEXT NOT NULL,
            lookup_count INTEGER NOT NULL DEFAULT 1,
            is_saved INTEGER NOT NULL DEFAULT 0,
            first_looked_up INTEGER NOT NULL,
            last_looked_up INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        INSERT OR IGNORE INTO app_settings (key, value) VALUES ('hotkey', 'ctrl+alt+d');",
    )
    .map_err(|e| format!("Failed to initialize user tables: {}", e))?;

    // Safe migrations if table previously existed without newer columns
    let _ = conn.execute(
        "ALTER TABLE word_history ADD COLUMN is_saved INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE word_history ADD COLUMN ipa TEXT NOT NULL DEFAULT ''",
        [],
    );

    // Create indexes AFTER columns are guaranteed to exist
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_word_history_saved ON word_history(is_saved, lookup_count DESC);
        CREATE INDEX IF NOT EXISTS idx_word_history_count ON word_history(lookup_count DESC);
        CREATE INDEX IF NOT EXISTS idx_word_history_last ON word_history(last_looked_up DESC);"
    ).map_err(|e| format!("Failed to create user indexes: {}", e))?;

    Ok(conn)
}

fn get_saved_hotkey() -> String {
    if let Ok(conn) = get_user_db() {
        conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'hotkey'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "ctrl+alt+d".to_string())
    } else {
        "ctrl+alt+d".to_string()
    }
}

fn save_hotkey(hotkey: &str) -> Result<(), String> {
    let conn = get_user_db()?;
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('hotkey', ?1)
         ON CONFLICT(key) DO UPDATE SET value = ?1;",
        params![hotkey],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn record_lookup(def: &Definition) -> (u32, bool) {
    if let Ok(conn) = get_user_db() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let json_meanings = serde_json::to_string(&def.meanings).unwrap_or_default();

        let _ = conn.execute(
            "INSERT INTO word_history (word, part_of_speech, ipa, definition_json, lookup_count, is_saved, first_looked_up, last_looked_up)
             VALUES (?1, ?2, ?3, ?4, 1, 0, ?5, ?5)
             ON CONFLICT(word) DO UPDATE SET
                 lookup_count = lookup_count + 1,
                 ipa = CASE WHEN ?3 != '' THEN ?3 ELSE ipa END,
                 last_looked_up = ?5;",
            params![def.word, def.part_of_speech, def.ipa, json_meanings, now],
        );

        // Retrieve current count and saved status
        conn.query_row(
            "SELECT lookup_count, is_saved FROM word_history WHERE word = ?1 COLLATE NOCASE",
            params![def.word],
            |row| Ok((row.get(0)?, row.get::<_, i32>(1)? != 0)),
        )
        .unwrap_or((1, false))
    } else {
        (1, false)
    }
}

#[tauri::command]
fn toggle_save_word(word: String) -> Result<bool, String> {
    let clean = word.trim();
    if clean.is_empty() {
        return Err("Word cannot be empty".to_string());
    }
    let conn = get_user_db()?;
    let exists: Option<i32> = conn
        .query_row(
            "SELECT is_saved FROM word_history WHERE word = ?1 COLLATE NOCASE",
            params![clean],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match exists {
        Some(saved_int) => {
            let new_state = saved_int == 0;
            conn.execute(
                "UPDATE word_history SET is_saved = ?1 WHERE word = ?2 COLLATE NOCASE",
                params![if new_state { 1 } else { 0 }, clean],
            )
            .map_err(|e| e.to_string())?;
            Ok(new_state)
        }
        None => {
            let def = lookup_word(clean.to_string())?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let json_meanings = serde_json::to_string(&def.meanings).unwrap_or_default();
            conn.execute(
                "INSERT INTO word_history (word, part_of_speech, ipa, definition_json, lookup_count, is_saved, first_looked_up, last_looked_up)
                 VALUES (?1, ?2, ?3, ?4, 1, 1, ?5, ?5)
                 ON CONFLICT(word) DO UPDATE SET is_saved = 1;",
                params![def.word, def.part_of_speech, def.ipa, json_meanings, now],
            ).map_err(|e| e.to_string())?;
            Ok(true)
        }
    }
}

#[tauri::command]
fn get_word_history() -> Result<Vec<WordHistoryItem>, String> {
    let conn = get_user_db()?;
    let mut stmt = conn
        .prepare("SELECT word, part_of_speech, ipa, definition_json, lookup_count, is_saved, last_looked_up FROM word_history ORDER BY is_saved DESC, lookup_count DESC, last_looked_up DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let word: String = row.get(0)?;
            let pos: String = row.get(1)?;
            let ipa: String = row.get(2)?;
            let json_str: String = row.get(3)?;
            let count: u32 = row.get(4)?;
            let is_saved: bool = row.get::<_, i32>(5)? != 0;
            let last: i64 = row.get(6)?;
            let meanings: Vec<String> = serde_json::from_str(&json_str).unwrap_or_default();
            Ok(WordHistoryItem {
                word,
                part_of_speech: pos,
                ipa,
                meanings,
                lookup_count: count,
                is_saved,
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
    conn.execute(
        "DELETE FROM word_history WHERE word = ?1 COLLATE NOCASE",
        params![word],
    )
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

    let fetch_entry = |target: &str| -> Result<Option<Definition>, String> {
        conn.query_row(
            "SELECT word, part_of_speech, ipa, meanings FROM entries WHERE word = ?1 COLLATE NOCASE",
            params![target],
            |row| {
                let w: String = row.get(0)?;
                let pos: String = row.get(1)?;
                let ipa: String = row.get(2)?;
                let m_json: String = row.get(3)?;
                let meanings: Vec<String> = serde_json::from_str(&m_json).unwrap_or_default();
                Ok(Definition {
                    word: w,
                    part_of_speech: pos,
                    ipa,
                    meanings,
                    lookup_count: 1,
                    is_saved: false,
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
        let (count, is_saved) = record_lookup(def);
        def.lookup_count = count;
        def.is_saved = is_saved;
    }

    println!("Lookup for '{}' took {:?}", clean_word, start.elapsed());

    result.ok_or_else(|| format!("'{}' not found in dictionary", clean_word))
}

#[tauri::command]
fn get_current_hotkey() -> String {
    get_saved_hotkey()
}

#[tauri::command]
fn set_current_hotkey(app: tauri::AppHandle, new_hotkey: String) -> Result<String, String> {
    let clean = new_hotkey.trim().to_lowercase();
    let new_shortcut: Shortcut = clean
        .parse()
        .map_err(|e| format!("Invalid shortcut: {:?}", e))?;

    let old_key = get_saved_hotkey();
    if let Ok(old_shortcut) = old_key.parse::<Shortcut>() {
        let _ = app.global_shortcut().unregister(old_shortcut);
    }

    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(new_shortcut, move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                handle_hotkey_trigger(&app_handle);
            }
        })
        .map_err(|e| format!("Failed to register shortcut '{}': {:?}", clean, e))?;

    save_hotkey(&clean)?;
    Ok(clean)
}

#[tauri::command]
fn resize_window(window: tauri::WebviewWindow, expanded: bool) -> Result<(), String> {
    if expanded {
        let _ = window.set_size(Size::Logical(LogicalSize {
            width: 720.0,
            height: 520.0,
        }));
        let _ = window.center();
    } else {
        let _ = window.set_size(Size::Logical(LogicalSize {
            width: 380.0,
            height: 340.0,
        }));
    }
    Ok(())
}

#[tauri::command]
fn speak_word(word: String) -> Result<(), String> {
    let clean = word.trim().to_string();
    if clean.is_empty() || clean.len() > 80 {
        return Err("Invalid word for speech".to_string());
    }

    #[cfg(target_os = "windows")]
    std::thread::spawn(move || {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Speak($env:LOOKMEUP_SPEAK_WORD);",
            ])
            .env("LOOKMEUP_SPEAK_WORD", &clean)
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    });

    Ok(())
}

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

fn position_popup_near_cursor(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    // Ensure compact popup size before computing bounds
    let _ = window.set_size(Size::Logical(LogicalSize {
        width: 380.0,
        height: 340.0,
    }));

    let Ok(pos) = app.cursor_position() else {
        return;
    };

    let scale = window.scale_factor().unwrap_or(1.0);
    let win_w = (380.0 * scale).round() as i32;
    let win_h = (340.0 * scale).round() as i32;
    let offset = (12.0 * scale).round() as i32;
    let bottom_pad = (48.0 * scale).round() as i32; // Reserve space for Windows taskbar

    // Find the monitor containing the cursor (supports multi-monitor setups)
    let cursor_x = pos.x.round() as i32;
    let cursor_y = pos.y.round() as i32;

    let active_monitor = app
        .available_monitors()
        .ok()
        .and_then(|monitors| {
            monitors.into_iter().find(|m| {
                let mp = m.position();
                let ms = m.size();
                cursor_x >= mp.x
                    && cursor_x < mp.x + ms.width as i32
                    && cursor_y >= mp.y
                    && cursor_y < mp.y + ms.height as i32
            })
        })
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());

    let (x, y) = if let Some(mon) = active_monitor {
        let mp = mon.position();
        let ms = mon.size();
        let mon_left = mp.x;
        let mon_top = mp.y;
        let mon_right = mp.x + ms.width as i32;
        let mon_bottom = mp.y + ms.height as i32;

        // Flip to left of cursor if too close to right edge, then clamp within monitor
        let mut target_x = cursor_x + offset;
        if target_x + win_w > mon_right - offset {
            target_x = cursor_x - win_w - offset;
        }
        target_x = target_x.clamp(mon_left + offset, (mon_right - win_w - offset).max(mon_left + offset));

        // Flip above cursor if too close to bottom edge/taskbar, then clamp within monitor
        let mut target_y = cursor_y + offset;
        if target_y + win_h > mon_bottom - bottom_pad {
            target_y = cursor_y - win_h - offset;
        }
        target_y = target_y.clamp(mon_top + offset, (mon_bottom - win_h - bottom_pad).max(mon_top + offset));

        (target_x, target_y)
    } else {
        ((cursor_x + offset).max(10), (cursor_y + offset).max(10))
    };

    let _ = window.set_position(Position::Physical(PhysicalPosition { x, y }));
}

fn handle_hotkey_trigger(app: &tauri::AppHandle) {
    #[cfg(target_os = "windows")]
    let _copied = simulate_ctrl_c();

    if let Some(window) = app.get_webview_window("main") {
        // Position window near cursor, clamped within active monitor bounds
        position_popup_near_cursor(app, &window);

        // Read text from clipboard
        let captured = if let Ok(text) = app.clipboard().read_text() {
            text.trim().to_string()
        } else {
            String::new()
        };

        // Extract first clean word
        let clean_word = captured
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '\'')
            .to_string();

        let payload = if !clean_word.is_empty() && clean_word.len() <= 40 {
            lookup_word(clean_word)
        } else {
            Err("No word selected. Type a word below:".to_string())
        };

        let _ = window.emit("show-definition", payload);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // 1. Register saved or default global shortcut
            let saved_key = get_saved_hotkey();
            let shortcut: Shortcut = saved_key
                .parse()
                .unwrap_or_else(|_| "ctrl+alt+d".parse().unwrap());
            let app_handle = app.handle().clone();
            let _ = app
                .global_shortcut()
                .on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        handle_hotkey_trigger(&app_handle);
                    }
                });

            // 2. Build System Tray
            let lookup_i = MenuItemBuilder::with_id("lookup", "Look Up Word...").build(app)?;
            let wordbank_i =
                MenuItemBuilder::with_id("wordbank", "Word Bank & Saved Words").build(app)?;
            let quit_i = MenuItemBuilder::with_id("quit", "Quit LookMeUp").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&lookup_i, &wordbank_i, &quit_i])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "wordbank" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("open-wordbank", ());
                            let _ = window.set_size(Size::Logical(LogicalSize {
                                width: 720.0,
                                height: 520.0,
                            }));
                            let _ = window.center();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "lookup" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("open-search", ());
                            let _ = window.set_size(Size::Logical(LogicalSize {
                                width: 380.0,
                                height: 340.0,
                            }));
                            let _ = window.center();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lookup_word,
            hide_window,
            get_word_history,
            delete_history_word,
            toggle_save_word,
            get_current_hotkey,
            set_current_hotkey,
            resize_window,
            speak_word
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
