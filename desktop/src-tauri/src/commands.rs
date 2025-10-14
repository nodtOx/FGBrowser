use crate::crawler::{FitGirlCrawler, GameRepack};
use crate::database::{AppSettings, Database, Game, GameDetails, DatabaseStats, CategoryWithCount};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use arboard::Clipboard;
use std::fs;

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
pub async fn get_disk_info() -> Result<DiskInfo, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Get disk info using wmic command
        let output = Command::new("wmic")
            .args(&["logicaldisk", "where", "size>0", "get", "size,freespace", "/format:csv"])
            .output()
            .map_err(|e| e.to_string())?;
            
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        let mut total = 0u64;
        let mut free = 0u64;
        
        for line in lines.iter().skip(1) { // Skip header
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                if let (Ok(size), Ok(freespace)) = (parts[2].parse::<u64>(), parts[3].parse::<u64>()) {
                    total += size;
                    free += freespace;
                }
            }
        }
        
        Ok(DiskInfo {
            total,
            free,
            used: total - free,
        })
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback for non-Windows systems
        Ok(DiskInfo {
            total: 0,
            free: 0,
            used: 0,
        })
    }
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

fn save_repacks_to_db(repacks: &[GameRepack], db_path: &PathBuf) -> anyhow::Result<()> {
    let db = Database::new(db_path.clone())?;
    
    // Process each repack individually with error handling
    let mut saved_count = 0;
    
    for repack in repacks {
        // Use a closure to handle individual game saves
        let save_result = (|| -> anyhow::Result<()> {
            // Parse size from repack_size for the integer column
            let parsed_size = parse_size_to_mb(&repack.repack_size);
            
            // Insert or update repack
            db.conn.execute(
                "INSERT INTO repacks (title, genres_tags, company, languages, original_size, repack_size, size, url, date, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)
                 ON CONFLICT(url) DO UPDATE SET
                    title = excluded.title,
                    genres_tags = excluded.genres_tags,
                    company = excluded.company,
                    languages = excluded.languages,
                    original_size = excluded.original_size,
                    repack_size = excluded.repack_size,
                    size = excluded.size,
                    date = excluded.date,
                    updated_at = CURRENT_TIMESTAMP",
                (&repack.title, &repack.genres_tags, &repack.company, &repack.languages, 
                 &repack.original_size, &repack.repack_size, &parsed_size, &repack.url, &repack.date),
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

