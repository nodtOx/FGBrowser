mod commands;
mod database;

use commands::{
    copy_to_clipboard, get_all_games, get_database_stats, get_disk_info, get_game_details,
    open_magnet_link, search_games, AppState,
};
use std::path::PathBuf;
use std::sync::Mutex;

fn find_database() -> PathBuf {
    // Try multiple locations to find the database
    let locations = vec![
        // In desktop folder (dev mode)
        PathBuf::from("../repacks.db"),
        // In project root (production)
        PathBuf::from("../../repacks.db"),
        // Relative to current exe
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("repacks.db")))
            .unwrap_or_default(),
        // In workspace root
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("repacks.db"),
    ];

    for path in locations {
        if path.exists() {
            println!("Found database at: {:?}", path);
            return path;
        }
    }

    // Default fallback
    println!("Database not found in standard locations, using default path");
    PathBuf::from("../repacks.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = find_database();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db_path: Mutex::new(db_path),
        })
        .invoke_handler(tauri::generate_handler![
            search_games,
            get_all_games,
            get_game_details,
            get_database_stats,
            open_magnet_link,
            copy_to_clipboard,
            get_disk_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
