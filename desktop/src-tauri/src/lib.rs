pub mod commands;
mod constants;
pub mod crawler;
pub mod database;
pub mod image_cache;

use commands::{
    // Game commands
    search_games, get_all_games, get_game_details, get_database_stats,
    get_categories_with_counts, get_categories_for_filtered_games, get_categories_for_search,
    get_categories_for_size_and_time_filtered_games, get_categories_for_size_filtered_games,
    get_categories_for_time_filtered_games, get_games_by_categories_and_size,
    get_games_by_categories_and_time, get_games_by_categories_size_and_time,
    get_games_by_category, get_games_by_date_range, get_games_by_multiple_categories,
    get_games_by_size_and_time, get_games_by_size_range, clear_category_cache, is_database_empty,
    // Crawler commands
    start_crawler, update_database,
    // Popular repacks commands
    fetch_popular_repacks, parse_popular_repacks_from_file, get_popular_repacks,
    get_popular_repacks_with_games, get_unseen_popular_count, get_total_unseen_popular_count,
    mark_popular_as_viewed, update_popular_repack_links, crawl_popular_games, crawl_single_popular_game,
    // Download commands
    get_downloads, add_download, pause_download, resume_download, remove_download, set_speed_limits,
    // Settings commands
    get_settings, save_settings, reset_database, download_database, check_database_exists,
    // Image commands
    get_cached_image, check_image_cached, cache_image_background, clear_image_cache, get_image_cache_size,
    // System commands
    get_app_constants, open_magnet_link, copy_to_clipboard, get_disk_info,
    select_download_folder, open_download_folder,
    // AppState
    AppState,
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
            get_app_constants,
            search_games,
            get_all_games,
            get_game_details,
            get_database_stats,
            get_categories_with_counts,
            get_categories_for_filtered_games,
            get_categories_for_time_filtered_games,
            get_categories_for_size_filtered_games,
            get_categories_for_size_and_time_filtered_games,
            get_categories_for_search,
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
            check_database_exists,
            download_database,
            open_magnet_link,
            copy_to_clipboard,
            get_disk_info,
            start_crawler,
            update_database,
            get_settings,
            save_settings,
            reset_database,
            fetch_popular_repacks,
            parse_popular_repacks_from_file,
            get_popular_repacks,
            get_popular_repacks_with_games,
            update_popular_repack_links,
            get_unseen_popular_count,
            get_total_unseen_popular_count,
            mark_popular_as_viewed,
            crawl_popular_games,
            crawl_single_popular_game,
            get_downloads,
            add_download,
            pause_download,
            resume_download,
            remove_download,
            set_speed_limits,
            select_download_folder,
            open_download_folder,
            get_cached_image,
            check_image_cached,
            cache_image_background,
            clear_image_cache,
            get_image_cache_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
