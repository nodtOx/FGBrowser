use crate::constants::{AppConstants, DATABASE_URL};
use crate::crawler::{FitGirlCrawler, GameRepack, clean_game_title};
use crate::database::{AppSettings, Database, Download, Game, GameDetails, DatabaseStats, CategoryWithCount};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use arboard::Clipboard;
use std::fs;
use std::io::Write;

/// Parse size string to MB (integer)
/// Handles patterns like:
/// - "916 MB" -> Some(916)
/// - "1.1 GB" -> Some(1100) 
/// - "from 15.9 GB [Selective" -> Some(15900)
/// - "916 MB/1.1 GB" -> Some(916) (takes smaller value)
/// - "1.1/1.3 GB" -> Some(1100) (takes smaller value)
fn parse_size_to_mb(size_str: &Option<String>) -> Option<i64> {
    let original = size_str.as_ref()?.trim();
    
    // Remove common prefixes and suffixes
    let size_str = original
        .strip_prefix("from ")
        .unwrap_or(original);
    let size_str = size_str
        .strip_prefix("~")
        .unwrap_or(size_str);
    
    // Remove trailing brackets and content
    let size_str = size_str.split('[').next().unwrap_or(size_str).trim();
    
    // Handle different slash patterns
    if size_str.contains('/') {
        let parts: Vec<&str> = size_str.split('/').collect();
        if parts.len() >= 2 {
            // Handle cases like "1.1/1.3 GB" where unit is only at the end
            let first_part = parts[0].trim();
            let second_part = parts[1].trim();
            
            // Check if first part has no unit but second part does
            let first_size = if first_part.split_whitespace().count() == 1 && second_part.split_whitespace().count() >= 2 {
                // Extract unit from second part and apply to first
                let second_parts: Vec<&str> = second_part.split_whitespace().collect();
                if second_parts.len() >= 2 {
                    let unit = second_parts[1];
                    parse_single_size(&format!("{} {}", first_part, unit))
                } else {
                    None
                }
            } else {
                parse_single_size(first_part)
            };
            
            let second_size = parse_single_size(second_part);
            
            match (first_size, second_size) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                _ => {
                    println!("❌ SIZE PARSE FAILED: '{}'", original);
                    None
                }
            }
        } else {
            println!("❌ SIZE PARSE FAILED: '{}'", original);
            None
        }
    } else {
        // Single size
        let result = parse_single_size(size_str);
        if result.is_none() {
            println!("❌ SIZE PARSE FAILED: '{}'", original);
        }
        result
    }
}

/// Parse a single size string like "1.4 GB" or "916 MB"
fn parse_single_size(size_str: &str) -> Option<i64> {
    let parts: Vec<&str> = size_str.split_whitespace().collect();
    
    if parts.len() < 2 {
        return None;
    }
    
    let number_str = parts[0];
    let unit_str = parts[1];
    
    // Extract just the unit part (remove any trailing characters)
    let unit = if unit_str.starts_with("MB") {
        "MB"
    } else if unit_str.starts_with("GB") {
        "GB" 
    } else if unit_str.starts_with("TB") {
        "TB"
    } else {
        return None;
    };
    
    // Parse the number
    let number: f64 = number_str.parse().ok()?;
    
    // Convert to MB
    let mb = match unit {
        "MB" => number,
        "GB" => number * 1024.0,
        "TB" => number * 1024.0 * 1024.0,
        _ => return None,
    };
    
    Some(mb as i64)
}

pub struct AppState {
    pub db_path: Mutex<PathBuf>,
}

#[tauri::command]
pub async fn search_games(
    query: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.search_games(&query, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_games(
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_all_games(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_game_details(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<GameDetails, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_game_details(game_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, AppState>,
) -> Result<DatabaseStats, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_with_counts(
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_with_counts().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_filtered_games(
    selected_category_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_filtered_games(&selected_category_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_time_filtered_games(
    days_ago: i32,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_time_filtered_games(days_ago).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_size_filtered_games(
    min_size: Option<i64>,
    max_size: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_size_filtered_games(min_size, max_size).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_size_and_time_filtered_games(
    min_size: Option<i64>,
    max_size: Option<i64>,
    days_ago: i32,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_size_and_time_filtered_games(min_size, max_size, days_ago).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_search(
    search_query: String,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_search(&search_query).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_date_range(
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_date_range(days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_size_range(
    min_size: Option<i64>,
    max_size: Option<i64>,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_size_range(min_size, max_size, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_categories_and_size(
    category_ids: Vec<i64>,
    min_size: Option<i64>,
    max_size: Option<i64>,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_categories_and_size(&category_ids, min_size, max_size, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_categories_and_time(
    category_ids: Vec<i64>,
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_categories_and_time(&category_ids, days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_size_and_time(
    min_size: Option<i64>,
    max_size: Option<i64>,
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_size_and_time(min_size, max_size, days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_categories_size_and_time(
    category_ids: Vec<i64>,
    min_size: Option<i64>,
    max_size: Option<i64>,
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_categories_size_and_time(&category_ids, min_size, max_size, days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_category(
    category_id: i64,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_category(category_id, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_multiple_categories(
    category_ids: Vec<i64>,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_multiple_categories(&category_ids, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_category_cache() -> Result<(), String> {
    Database::clear_category_cache();
    println!("🧹 Category cache cleared");
    Ok(())
}

#[tauri::command]
pub async fn is_database_empty(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    
    // Check if file exists
    if !db_path.exists() {
        return Ok(true);
    }
    
    // Check if database has repacks table and any games
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // Check if repacks table exists
    let table_exists: Result<i64, _> = db.conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='repacks'",
        [],
        |row| row.get(0),
    );
    
    match table_exists {
        Ok(count) if count > 0 => {
            // Table exists, check if it has data
            let stats = db.get_stats().map_err(|e| e.to_string())?;
            Ok(stats.total_games == 0)
        }
        _ => {
            // Table doesn't exist, database is empty
            Ok(true)
        }
    }
}

#[tauri::command]
pub async fn open_magnet_link(magnet: String) -> Result<(), String> {
    // This will be handled by the system's default torrent client
    // Or we can implement our own torrent client here
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &magnet])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&magnet)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&magnet)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())
}

#[derive(Debug, serde::Serialize)]
pub struct DiskInfo {
    pub total: u64,
    pub free: u64,
    pub used: u64,
}

#[tauri::command]
pub async fn get_disk_info(state: State<'_, AppState>) -> Result<DiskInfo, String> {
    use sysinfo::Disks;
    use std::path::Path;
    
    let disks = Disks::new_with_refreshed_list();
    let db_path = state.db_path.lock().unwrap().clone();
    
    // Get the absolute path of the database directory
    let db_dir = db_path.parent()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    
    // Find the disk that contains the database directory
    let mut target_disk = None;
    for disk in disks.list() {
        let mount_point = disk.mount_point();
        // Check if the database path is on this disk
        if db_dir.starts_with(mount_point) {
            // Keep the most specific mount point (longest path)
            if let Some((_, current_mount)) = target_disk {
                if mount_point.as_os_str().len() > Path::new(current_mount).as_os_str().len() {
                    target_disk = Some((disk, mount_point));
                }
            } else {
                target_disk = Some((disk, mount_point));
            }
        }
    }
    
    // If we found a matching disk, use it; otherwise fall back to the first disk
    let disk = target_disk
        .map(|(d, _)| d)
        .or_else(|| disks.list().first())
        .ok_or("No disks found")?;
    
    let total = disk.total_space();
    let available = disk.available_space();
    let used = total.saturating_sub(available);
    
    Ok(DiskInfo {
        total,
        free: available,
        used,
    })
}

// Crawler commands

#[derive(Debug, serde::Serialize, Clone)]
pub struct CrawlProgress {
    pub current_page: u32,
    pub total_games: usize,
    pub status: String,
}

#[tauri::command]
pub async fn start_crawler(
    state: State<'_, AppState>,
) -> Result<CrawlProgress, String> {
    use std::time::Instant;
    
    // HARDCODED: Always crawl exactly 10 pages
    const MAX_PAGES: u32 = 10;
    
    let start_time = Instant::now();
    println!("🚀 CRAWLER STARTED - {} pages", MAX_PAGES);
    
    // Create crawler
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let db_path = state.db_path.lock().unwrap().clone();
    
    // Crawl pages and save incrementally
    let mut total_games = 0;
    let mut current_page = 1u32;
    
    loop {
        // Check if we've reached max_pages (HARDCODED)
        if current_page > MAX_PAGES {
            println!("✅ Completed {} pages", MAX_PAGES);
            break;
        }
        
        // Crawl single page (includes delay)
        match crawler.crawl_page(current_page).await {
            Ok(repacks) => {
                if repacks.is_empty() {
                    println!("\nNo more content found at page {}", current_page);
                    break;
                }
                
                let count = repacks.len();
                
                // Save to database immediately after each page
                if let Err(e) = save_repacks_to_db(&repacks, &db_path) {
                    eprintln!("Error saving page {}: {}", current_page, e);
                    // Continue anyway - we've saved what we could
                } else {
                    total_games += count;
                    println!("📄 Page {}: {} games", current_page, count);
                }
            }
            Err(e) => {
                eprintln!("Error crawling page {}: {}", current_page, e);
                // Don't break immediately, return what we have
                break;
            }
        }
        
        current_page += 1;
    }
    
    let duration = start_time.elapsed();
    let seconds = duration.as_secs();
    let millis = duration.subsec_millis();
    
    println!("\n{}", "=".repeat(80));
    println!("CRAWLER COMPLETED");
    println!("Total Games: {}", total_games);
    println!("Pages Crawled: {}", current_page - 1);
    println!("Time Taken: {}.{:03}s", seconds, millis);
    println!("Average: {:.2}s per page", duration.as_secs_f64() / (current_page - 1) as f64);
    println!("{}", "=".repeat(80));
    
    Ok(CrawlProgress {
        current_page: current_page - 1,
        total_games,
        status: "Completed".to_string(),
    })
}

pub fn save_repacks_to_db(repacks: &[GameRepack], db_path: &PathBuf) -> anyhow::Result<()> {
    let db = Database::new(db_path.clone())?;
    
    // Process each repack individually with error handling
    let mut saved_count = 0;
    
    for repack in repacks {
        // Use a closure to handle individual game saves
        let save_result = (|| -> anyhow::Result<()> {
            // Parse size from repack_size for the integer column
            let parsed_size = parse_size_to_mb(&repack.repack_size);
            
            // Generate clean name
            let clean_name = clean_game_title(&repack.title);
            
            // Insert or update repack
            db.conn.execute(
                "INSERT INTO repacks (title, clean_name, genres_tags, company, languages, original_size, repack_size, size, url, date, image_url, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)
                 ON CONFLICT(url) DO UPDATE SET
                    title = excluded.title,
                    clean_name = excluded.clean_name,
                    genres_tags = excluded.genres_tags,
                    company = excluded.company,
                    languages = excluded.languages,
                    original_size = excluded.original_size,
                    repack_size = excluded.repack_size,
                    size = excluded.size,
                    date = excluded.date,
                    image_url = excluded.image_url,
                    updated_at = CURRENT_TIMESTAMP",
                (&repack.title, &clean_name, &repack.genres_tags, &repack.company, &repack.languages, 
                 &repack.original_size, &repack.repack_size, &parsed_size, &repack.url, &repack.date, &repack.image_url),
            )?;
            
            // Get repack_id
            let repack_id: i64 = db.conn.query_row(
                "SELECT id FROM repacks WHERE url = ?1",
                [&repack.url],
                |row| row.get(0),
            )?;
            
            // Insert magnet links
            for magnet in &repack.magnet_links {
                db.conn.execute(
                    "INSERT INTO magnet_links (repack_id, source, magnet)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(repack_id, source) DO UPDATE SET magnet = excluded.magnet",
                    (repack_id, &magnet.source, &magnet.magnet),
                )?;
            }
            
            // Insert categories if genres_tags is present
            if let Some(genres_tags) = &repack.genres_tags {
                if !genres_tags.trim().is_empty() {
                    let categories: Vec<String> = genres_tags
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    
                    for category_name in categories {
                        // Insert category if not exists
                        db.conn.execute(
                            "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
                            [&category_name],
                        )?;
                        
                        // Get category ID
                        let category_id: i64 = db.conn.query_row(
                            "SELECT id FROM categories WHERE name = ?1",
                            [&category_name],
                            |row| row.get(0),
                        )?;
                        
                        // Insert game-category relationship
                        db.conn.execute(
                            "INSERT OR IGNORE INTO game_categories (repack_id, category_id) VALUES (?1, ?2)",
                            [repack_id, category_id],
                        )?;
                    }
                }
            }
            
            Ok(())
        })();
        
        match save_result {
            Ok(_) => saved_count += 1,
            Err(e) => {
                eprintln!("Failed to save repack '{}': {}", repack.title, e);
                // Continue with next repack instead of failing completely
            }
        }
    }
    
    if saved_count > 0 {
        println!("Saved {}/{} repacks to database", saved_count, repacks.len());
        // Clear category cache since new games were added
        Database::clear_category_cache();
        println!("🧹 Category cache cleared after adding {} games", saved_count);
    }
    
    Ok(())
}

// Settings commands

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_settings().map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_database(state: State<'_, AppState>) -> Result<(), String> {
    let db_path = state.db_path.lock().unwrap().clone();
    
    if db_path.exists() {
        fs::remove_file(&db_path).map_err(|e| format!("Failed to delete database: {}", e))?;
        println!("Database deleted: {:?}", db_path);
        
        // Clear cache since database was reset
        Database::clear_category_cache();
        println!("🧹 Category cache cleared after database reset");
    }
    
    Ok(())
}

#[tauri::command]
pub async fn update_database(
    state: State<'_, AppState>,
) -> Result<CrawlProgress, String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path.clone()).map_err(|e| e.to_string())?;
    
    // Get the latest game date
    let latest_date = db.get_latest_game_date().map_err(|e| e.to_string())?;
    
    if latest_date.is_none() {
        return Ok(CrawlProgress {
            current_page: 0,
            total_games: 0,
            status: "No existing data".to_string(),
        });
    }
    
    println!("\n{}", "=".repeat(80));
    println!("DATABASE UPDATE STARTED");
    println!("Latest game date: {:?}", latest_date);
    println!("{}", "=".repeat(80));
    
    // Create crawler
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    
    let mut total_new_games = 0;
    let mut current_page = 1u32;
    
    // Crawl pages until we find games older than our latest
    loop {
        match crawler.crawl_page(current_page).await {
            Ok(repacks) => {
                if repacks.is_empty() {
                    break;
                }
                
                // Filter out games that already exist in database by URL
                // This is more reliable than date comparison
                let new_repacks: Vec<_> = repacks
                    .into_iter()
                    .filter(|r| {
                        // Check if this URL already exists in database
                        let exists: Result<i64, _> = db.conn.query_row(
                            "SELECT COUNT(*) FROM repacks WHERE url = ?1",
                            [&r.url],
                            |row| row.get(0),
                        );
                        
                        match exists {
                            Ok(count) => count == 0, // Include if doesn't exist
                            Err(_) => true, // Include on error
                        }
                    })
                    .collect();
                
                if new_repacks.is_empty() {
                    println!("No new games found on page {}, stopping update", current_page);
                    break;
                }
                
                let count = new_repacks.len();
                
                // Save new games to database
                if let Err(e) = save_repacks_to_db(&new_repacks, &db_path) {
                    eprintln!("Error saving page {}: {}", current_page, e);
                } else {
                    total_new_games += count;
                    println!(
                        "[UPDATE] Page {}: Found {} new games (Total new: {})",
                        current_page, count, total_new_games
                    );
                }
            }
            Err(e) => {
                eprintln!("Error crawling page {}: {}", current_page, e);
                break;
            }
        }
        
        current_page += 1;
        
        // Safety limit - don't update more than 10 pages
        if current_page > 10 {
            println!("Reached 10 page limit for update");
            break;
        }
    }
    
    let duration = start_time.elapsed();
    
    println!("\n{}", "=".repeat(80));
    println!("DATABASE UPDATE COMPLETED");
    println!("New Games: {}", total_new_games);
    println!("Time Taken: {:.2}s", duration.as_secs_f64());
    println!("{}", "=".repeat(80));
    
    Ok(CrawlProgress {
        current_page: current_page - 1,
        total_games: total_new_games,
        status: "Updated".to_string(),
    })
}

// Popular repacks commands

#[tauri::command]
pub async fn fetch_popular_repacks(
    period: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    use crate::crawler::FitGirlCrawler;
    use crate::database::Database;
    
    println!("🌟 Fetching popular repacks ({}) from website...", period);
    
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let popular_entries = crawler.fetch_popular_repacks(&period).await.map_err(|e| e.to_string())?;
    
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // Clear existing popular repacks for this period
    db.clear_popular_repacks(&period).map_err(|e| e.to_string())?;
    
    // Save new popular repacks with rank based on order (1, 2, 3...)
    let mut saved_count = 0;
    for (index, entry) in popular_entries.iter().enumerate() {
        let rank = (index + 1) as i32;
        match db.save_popular_repack(
            &entry.url, 
            &entry.title, 
            entry.image_url.as_deref(), 
            rank,
            &period
        ) {
            Ok(_) => saved_count += 1,
            Err(e) => eprintln!("Failed to save popular repack '{}': {}", entry.title, e),
        }
    }
    
    // Update links to existing games for this period
    let linked_count = db.update_popular_repack_links(Some(&period)).map_err(|e| e.to_string())?;
    
    println!("✅ Saved {} popular repacks ({}) ({} linked to existing games)", saved_count, period, linked_count);
    
    Ok(saved_count)
}

#[tauri::command]
pub async fn parse_popular_repacks_from_file(
    file_path: String,
    period: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    use crate::crawler::FitGirlCrawler;
    use crate::database::Database;
    
    println!("📄 Parsing popular repacks from file: {}", file_path);
    
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let popular_entries = crawler.parse_popular_repacks_from_file(&file_path).map_err(|e| e.to_string())?;
    
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // Clear existing popular repacks for this period
    db.clear_popular_repacks(&period).map_err(|e| e.to_string())?;
    
    // Save new popular repacks with rank based on order (1, 2, 3...)
    let mut saved_count = 0;
    for (index, entry) in popular_entries.iter().enumerate() {
        let rank = (index + 1) as i32;
        match db.save_popular_repack(
            &entry.url, 
            &entry.title, 
            entry.image_url.as_deref(), 
            rank,
            &period
        ) {
            Ok(_) => saved_count += 1,
            Err(e) => eprintln!("Failed to save popular repack '{}': {}", entry.title, e),
        }
    }
    
    // Update links to existing games for this period
    let linked_count = db.update_popular_repack_links(Some(&period)).map_err(|e| e.to_string())?;
    
    println!("✅ Saved {} popular repacks ({}) ({} linked to existing games)", saved_count, period, linked_count);
    
    Ok(saved_count)
}

#[tauri::command]
pub async fn get_popular_repacks(
    period: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<crate::database::PopularRepack>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_popular_repacks(&period, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_popular_repacks_with_games(
    period: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<crate::database::PopularRepackWithGame>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_popular_repacks_with_games(&period, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_unseen_popular_count(
    period: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    let settings = db.get_settings().map_err(|e| e.to_string())?;
    
    let last_viewed = match period.as_str() {
        "month" => settings.popular_month_last_viewed.as_deref(),
        "year" => settings.popular_year_last_viewed.as_deref(),
        "award" => settings.popular_award_last_viewed.as_deref(),
        _ => None,
    };
    
    db.get_unseen_popular_count(&period, last_viewed).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_total_unseen_popular_count(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    let settings = db.get_settings().map_err(|e| e.to_string())?;
    
    db.get_total_unseen_popular_count(
        settings.popular_month_last_viewed.as_deref(),
        settings.popular_year_last_viewed.as_deref(),
        settings.popular_award_last_viewed.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_popular_as_viewed(
    period: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    let mut settings = db.get_settings().map_err(|e| e.to_string())?;
    
    // Get current timestamp in ISO 8601 format
    let now = chrono::Utc::now().to_rfc3339();
    
    // Update the appropriate timestamp
    match period.as_str() {
        "month" => settings.popular_month_last_viewed = Some(now),
        "year" => settings.popular_year_last_viewed = Some(now),
        "award" => settings.popular_award_last_viewed = Some(now),
        _ => return Err("Invalid period".to_string()),
    }
    
    db.save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_popular_repack_links(
    period: Option<String>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    let count = db.update_popular_repack_links(period.as_deref()).map_err(|e| e.to_string())?;
    println!("🔗 Updated links for {} popular repacks", count);
    Ok(count)
}

#[tauri::command]
pub async fn crawl_popular_games(
    period: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    use crate::crawler::FitGirlCrawler;
    use crate::database::Database;
    
    println!("🎮 Crawling popular games data ({})...", period);
    
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path.clone()).map_err(|e| e.to_string())?;
    
    // Get popular repacks for the specified period
    let popular_repacks = db.get_popular_repacks(&period, 200).map_err(|e| e.to_string())?;
    
    if popular_repacks.is_empty() {
        return Err(format!("No popular repacks ({}) found. Fetch popular repacks first.", period));
    }
    
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let mut crawled_count = 0;
    let mut skipped_count = 0;
    
    for popular in &popular_repacks {
        // Skip if already linked to a game in database
        if popular.repack_id.is_some() {
            skipped_count += 1;
            continue;
        }
        
        println!("  Crawling: {}", popular.title);
        
        // Crawl the game page
        match crawler.crawl_single_game(&popular.url).await {
            Ok(Some(game_repack)) => {
                // Save to database
                if let Err(e) = save_repacks_to_db(&[game_repack], &db_path) {
                    eprintln!("    ❌ Failed to save: {}", e);
                } else {
                    crawled_count += 1;
                    println!("    ✓ Saved");
                }
            }
            Ok(None) => {
                eprintln!("    ⚠ Could not extract game data");
            }
            Err(e) => {
                eprintln!("    ❌ Crawl error: {}", e);
            }
        }
    }
    
    // Update links after crawling for this period
    let linked = db.update_popular_repack_links(Some(&period)).map_err(|e| e.to_string())?;
    
    println!("\n✅ Crawled {} new games ({}), skipped {} existing, linked {}", 
             crawled_count, period, skipped_count, linked);
    
    Ok(crawled_count)
}

#[tauri::command]
pub async fn crawl_single_popular_game(
    url: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    use crate::crawler::FitGirlCrawler;
    
    println!("🎮 Crawling single game: {}", url);
    
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let db_path = state.db_path.lock().unwrap().clone();
    
    match crawler.crawl_single_game(&url).await {
        Ok(Some(game_repack)) => {
            // Save to database
            save_repacks_to_db(&[game_repack], &db_path).map_err(|e| e.to_string())?;
            
            // Update links for all periods
            let db = Database::new(db_path).map_err(|e| e.to_string())?;
            db.update_popular_repack_links(None).map_err(|e| e.to_string())?;
            
            println!("✅ Game crawled and saved");
            Ok(true)
        }
        Ok(None) => {
            println!("⚠ Could not extract game data");
            Ok(false)
        }
        Err(e) => {
            eprintln!("❌ Crawl error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn get_app_constants() -> AppConstants {
    AppConstants::new()
}

// Database download command

#[derive(Debug, serde::Serialize, Clone)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub status: String,
}

#[tauri::command]
pub async fn download_database(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    let db_path = state.db_path.lock().unwrap().clone();
    
    println!("\n{}", "=".repeat(80));
    println!("DATABASE DOWNLOAD STARTED");
    println!("Source: {}", DATABASE_URL);
    println!("Target: {:?}", db_path);
    println!("{}", "=".repeat(80));
    
    // Check if database already exists
    if db_path.exists() {
        // Check if it has data
        match Database::new(db_path.clone()) {
            Ok(db) => {
                if let Ok(stats) = db.get_stats() {
                    if stats.total_games > 0 {
                        println!("Database already exists with {} games, skipping download", stats.total_games);
                        return Ok(false);
                    }
                }
            }
            Err(_) => {
                // Database file exists but is corrupted, delete it
                println!("Existing database file is corrupted, removing...");
                let _ = fs::remove_file(&db_path);
            }
        }
    }
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    // Download the database
    println!("Downloading database from server...");
    
    let response = reqwest::get(DATABASE_URL)
        .await
        .map_err(|e| format!("Failed to download database: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }
    
    let total_size = response.content_length();
    if let Some(size) = total_size {
        println!("Database size: {:.2} MB", size as f64 / 1024.0 / 1024.0);
    }
    
    let bytes = response.bytes()
        .await
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
                    Ok(true)
                }
                Err(e) => {
                    // Database downloaded but can't read stats - might be empty but valid
                    println!("Warning: Downloaded database but couldn't read stats: {}", e);
                    Ok(true)
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

#[tauri::command]
pub async fn check_database_exists(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    Ok(db_path.exists())
}

// Download Management Commands

#[tauri::command]
pub async fn get_downloads(state: State<'_, AppState>) -> Result<Vec<Download>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_all_downloads().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_download(
    magnet: String,
    repack_id: i64,
    save_path: String,
    state: State<'_, AppState>,
) -> Result<Download, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // Extract info hash from magnet link
    let info_hash = extract_info_hash(&magnet).ok_or("Invalid magnet link")?;
    
    // Get game details for title
    let game = db.get_game_details(repack_id).map_err(|e| e.to_string())?;
    let game_title = game.game.clean_name.as_ref().unwrap_or(&game.game.title);
    
    // Create download record
    let _id = db.create_download(repack_id, game_title, &magnet, &info_hash, &save_path)
        .map_err(|e| e.to_string())?;
    
    // TODO: Actually start the torrent download here
    // For now, just create the database record
    
    // Return the created download
    db.get_download_by_info_hash(&info_hash)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Download created but not found".to_string())
}

#[tauri::command]
pub async fn pause_download(
    info_hash: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // TODO: Pause the actual torrent
    
    db.update_download_status(&info_hash, "paused", None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_download(
    info_hash: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // TODO: Resume the actual torrent
    
    db.update_download_status(&info_hash, "downloading", None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_download(
    info_hash: String,
    delete_files: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // Get download info before deleting
    let download = db.get_download_by_info_hash(&info_hash)
        .map_err(|e| e.to_string())?
        .ok_or("Download not found")?;
    
    // TODO: Stop and remove the actual torrent
    
    // Delete files if requested
    if delete_files {
        let _ = fs::remove_dir_all(&download.save_path);
    }
    
    db.delete_download(&info_hash).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_speed_limits(
    download_kbps: i32,
    upload_kbps: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Apply speed limits to torrent client
    
    // For now, just update settings
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    let mut settings = db.get_settings().unwrap_or_default();
    settings.max_download_speed = download_kbps;
    settings.max_upload_speed = upload_kbps;
    db.save_settings(&settings).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn select_download_folder() -> Result<Option<String>, String> {
    use rfd::FileDialog;
    
    let folder = FileDialog::new()
        .set_title("Select Download Location")
        .pick_folder();
    
    Ok(folder.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn open_download_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// Helper function to extract info hash from magnet link
fn extract_info_hash(magnet: &str) -> Option<String> {
    // magnet:?xt=urn:btih:INFO_HASH&...
    if !magnet.starts_with("magnet:?") {
        return None;
    }
    
    for part in magnet.split('&') {
        if let Some(xt) = part.strip_prefix("xt=urn:btih:") {
            return Some(xt.to_string());
        }
        if part.contains("xt=urn:btih:") {
            if let Some(hash) = part.split("xt=urn:btih:").nth(1) {
                return Some(hash.to_string());
            }
        }
    }
    
    None
}

