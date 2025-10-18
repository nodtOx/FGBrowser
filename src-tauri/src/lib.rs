pub mod commands;
mod constants;
pub mod crawler;
pub mod database;
pub mod image_cache;
mod telemetry;

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
    // Telemetry commands
    track_app_launch, track_feature_usage, track_crawler_run, track_error, is_telemetry_enabled, test_sentry_integration,
    // AppState
    AppState,
};
use commands::database_service::SqliteDatabaseService;
use database::Database;
use constants::DATABASE_URL;
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use std::io::Write;
use rusqlite;

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

fn should_download_database(db_path: &PathBuf) -> bool {
    if !db_path.exists() {
        return true;
    }
    
    // Check if database file is empty (0 bytes) or very small
    if let Ok(metadata) = fs::metadata(db_path) {
        if metadata.len() < 1024 {  // Less than 1KB - probably empty or just schema
            return true;
        }
    }
    
    // Check if database has data by opening read-only connection
    // Use a direct SQL query without creating tables
    match rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
    ) {
        Ok(conn) => {
            // Try to count games without creating tables
            match conn.query_row("SELECT COUNT(*) FROM repacks", [], |row| row.get::<_, i64>(0)) {
                Ok(count) => count == 0,
                Err(_) => true,  // Table doesn't exist or error reading
            }
        }
        Err(_) => true,  // Can't open database
    }
}

fn download_database_sync(db_path: &PathBuf) -> Result<(), String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    
    println!("\n{}", "=".repeat(80));
    println!("DATABASE DOWNLOAD STARTED (STARTUP)");
    println!("Source: {}", DATABASE_URL);
    println!("Target: {:?}", db_path);
    println!("{}", "=".repeat(80));
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    // Download the database using blocking reqwest
    println!("Downloading database from server...");
    
    let response = reqwest::blocking::get(DATABASE_URL)
        .map_err(|e| format!("Failed to download database: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }
    
    let total_size = response.content_length();
    if let Some(size) = total_size {
        println!("Database size: {:.2} MB", size as f64 / 1024.0 / 1024.0);
    }
    
    let bytes = response.bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    // Write to file
    let mut file = fs::File::create(&db_path)
        .map_err(|e| format!("Failed to create database file: {}", e))?;
    
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write database file: {}", e))?;
    
    let duration = start_time.elapsed();
    
    // Verify the downloaded database
    match Database::new(db_path.clone()) {
        Ok(db) => {
            match db.get_stats() {
                Ok(stats) => {
                    println!("\n{}", "=".repeat(80));
                    println!("DATABASE DOWNLOAD COMPLETED");
                    println!("Total Games: {}", stats.total_games);
                    println!("Size: {:.2} MB", bytes.len() as f64 / 1024.0 / 1024.0);
                    println!("Time Taken: {:.2}s", duration.as_secs_f64());
                    println!("{}", "=".repeat(80));
                    Ok(())
                }
                Err(e) => {
                    Err(format!("Downloaded database but couldn't read stats: {}", e))
                }
            }
        }
        Err(e) => {
            // Delete corrupted file
            let _ = fs::remove_file(&db_path);
            Err(format!("Downloaded database is invalid: {}", e))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = find_database();
    
    // Check if database needs to be downloaded BEFORE initializing connection
    if should_download_database(&db_path) {
        println!("Database not found or empty, downloading from server...");
        match download_database_sync(&db_path) {
            Ok(_) => {
                println!("✅ Database downloaded successfully during startup");
            }
            Err(e) => {
                eprintln!("❌ Failed to download database during startup: {}", e);
                eprintln!("The app will start but may not have game data.");
                eprintln!("You can try downloading again from the app settings.");
            }
        }
    }
    
    // Initialize the DatabaseService with shared connection
    // This follows Dependency Inversion Principle and improves performance
    let db_service = match SqliteDatabaseService::new(db_path.clone()) {
        Ok(service) => Arc::new(service),
        Err(e) => {
            eprintln!("Failed to initialize database service: {}", e);
            eprintln!("Database path: {:?}", db_path);
            std::process::exit(1);
        }
    };

    println!("✅ Database service initialized successfully");

    // Initialize telemetry
    telemetry::init_telemetry();

    // Clone db_path for commands that need it directly (reset/download/check)
    let db_path_for_commands = db_path.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            db_service,
        })
        .manage(db_path_for_commands)
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
            get_image_cache_size,
            track_app_launch,
            track_feature_usage,
            track_crawler_run,
            track_error,
            is_telemetry_enabled,
            test_sentry_integration
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
