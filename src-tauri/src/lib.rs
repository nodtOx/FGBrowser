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
    mark_all_games_as_seen, get_new_games_count,
    // Crawler commands
    start_crawler, update_database, save_repacks_to_db,
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
use commands::database_service::{SqliteDatabaseService, DatabaseService};
use database::Database;
use constants::DATABASE_URL;
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use std::io::Write;
use tauri::Manager;

#[cfg(not(target_os = "macos"))]
use tauri::{Emitter, menu::{Menu, MenuItem}, tray::{TrayIconBuilder, MouseButton, MouseButtonState}};
#[cfg(not(target_os = "macos"))]
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

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
    let mut file = fs::File::create(db_path)
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
            let _ = fs::remove_file(db_path);
            Err(format!("Downloaded database is invalid: {}", e))
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide Window", true, None::<&str>)?;
    let check_updates = MenuItem::with_id(app, "check_updates", "Check for Updates", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[
        &show_hide,
        &check_updates,
        &about,
        &tauri::menu::PredefinedMenuItem::separator(app)?,
        &quit,
    ])?;
    
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show_hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                "check_updates" => {
                    if let Some(window) = app.get_webview_window("main") {
                        // Emit event to frontend to trigger update check
                        let _ = window.emit("check-for-updates", ());
                    }
                }
                "about" => {
                    let version = app.package_info().version.to_string();
                    let message = format!(
                        "FGBrowser v{}\n\nGitHub Repository:\nhttps://github.com/nodtOx/FGBrowser\n\nBuilt with Tauri 2.0 + Svelte",
                        version
                    );
                    
                    let app_handle = app.clone();
                    app.dialog()
                        .message(message)
                        .title("About FGBrowser")
                        .kind(MessageDialogKind::Info)
                        .buttons(MessageDialogButtons::OkCancelCustom(
                            "Open GitHub".to_string(),
                            "Report Issue".to_string()
                        ))
                        .show(move |result| {
                            use tauri_plugin_opener::OpenerExt;
                            match result {
                                true => {
                                    // OK button (Open GitHub)
                                    let _ = app_handle.opener().open_url("https://github.com/nodtOx/FGBrowser", None::<&str>);
                                }
                                false => {
                                    // Cancel button (Report Issue)
                                    let _ = app_handle.opener().open_url("https://github.com/nodtOx/FGBrowser/issues/new", None::<&str>);
                                }
                            }
                        });
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
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

    // Clone db_service for background crawler
    let db_service_for_crawler = Arc::clone(&db_service);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            // Setup system tray (skip on macOS during development)
            #[cfg(not(target_os = "macos"))]
            setup_tray(app)?;
            
            // Handle window close event - hide to tray instead of closing
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent the window from closing
                    api.prevent_close();
                    // Hide the window instead
                    let _ = window_clone.hide();
                }
            });
            
            // Start background crawler task (runs every 10 minutes)
            let db_service_clone = Arc::clone(&db_service_for_crawler);
            tauri::async_runtime::spawn(async move {
                // Wait 10 minutes before first run (let app settle)
                std::thread::sleep(std::time::Duration::from_secs(600));
                
                loop {
                    println!("🔄 Background crawler: Checking for new games...");
                    
                    // Run update_database
                    match db_service_clone.get_latest_game_date() {
                        Ok(Some(_latest_date)) => {
                            // We have existing data, run incremental update
                            match crate::crawler::FitGirlCrawler::new() {
                                Ok(crawler) => {
                                    let mut current_page = 1u32;
                                    let mut total_new_games = 0;
                                    
                                    // Crawl until we find games we already have
                                    loop {
                                        match crawler.crawl_page(current_page).await {
                                            Ok(repacks) => {
                                                if repacks.is_empty() {
                                                    break;
                                                }
                                                
                                                // Filter out existing games
                                                let new_repacks: Vec<_> = repacks
                                                    .into_iter()
                                                    .filter(|r| {
                                                        match db_service_clone.check_url_exists(&r.url) {
                                                            Ok(exists) => !exists,
                                                            Err(_) => true,
                                                        }
                                                    })
                                                    .collect();
                                                
                                                if new_repacks.is_empty() {
                                                    break;
                                                }
                                                
                                                // Save new games
                                                match save_repacks_to_db(&new_repacks, &db_service_clone) {
                                                    Ok(_) => {
                                                        total_new_games += new_repacks.len();
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to save repacks: {}", e);
                                                    }
                                                }
                                                
                                                current_page += 1;
                                                
                                                // Limit to 5 pages per background check
                                                if current_page > 5 {
                                                    break;
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Background crawler error: {}", e);
                                                break;
                                            }
                                        }
                                    }
                                    
                                    if total_new_games > 0 {
                                        println!("✅ Background crawler: Found {} new games", total_new_games);
                                    } else {
                                        println!("✅ Background crawler: No new games found");
                                    }
                                }
                                Err(e) => eprintln!("Background crawler failed to initialize: {}", e),
                            }
                        }
                        Ok(None) => {
                            println!("⏭️  Background crawler: Skipping (no existing data)");
                        }
                        Err(e) => eprintln!("Background crawler error checking latest game: {}", e),
                    }
                    
                    // Wait 10 minutes before next run
                    std::thread::sleep(std::time::Duration::from_secs(600));
                }
            });
            
            Ok(())
        })
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
            mark_all_games_as_seen,
            get_new_games_count,
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
