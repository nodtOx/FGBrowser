mod commands;
mod crawler;
mod database;

use commands::{
    clear_category_cache, copy_to_clipboard, get_all_games, get_categories_with_counts, get_categories_for_filtered_games, get_categories_for_size_and_time_filtered_games, get_categories_for_size_filtered_games, get_categories_for_time_filtered_games, get_database_stats, get_disk_info, get_game_details,
    get_games_by_categories_and_size, get_games_by_categories_and_time, get_games_by_categories_size_and_time, get_games_by_category, get_games_by_date_range, get_games_by_multiple_categories, get_games_by_size_and_time, get_games_by_size_range, get_settings, is_database_empty, open_magnet_link, reset_database, save_settings, 
    search_games, start_crawler, update_database, AppState,
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
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db_path: Mutex::new(db_path),
        })
        .invoke_handler(tauri::generate_handler![
            search_games,
            get_all_games,
            get_game_details,
            get_database_stats,
            get_categories_with_counts,
            get_categories_for_filtered_games,
            get_categories_for_time_filtered_games,
            get_categories_for_size_filtered_games,
            get_categories_for_size_and_time_filtered_games,
            get_games_by_category,
            get_games_by_date_range,
            get_games_by_size_range,
            get_games_by_categories_and_size,
            get_games_by_categories_and_time,
            get_games_by_size_and_time,
            get_games_by_categories_size_and_time,
            get_games_by_multiple_categories,
            clear_category_cache,
            is_database_empty,
            open_magnet_link,
            copy_to_clipboard,
            get_disk_info,
            start_crawler,
            update_database,
            get_settings,
            save_settings,
            reset_database
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
