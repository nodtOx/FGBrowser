use crate::crawler::{FitGirlCrawler, GameRepack};
use crate::database::{AppSettings, Database, Game, GameDetails, DatabaseStats};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use arboard::Clipboard;
use std::fs;

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
    max_pages: Option<u32>,
    state: State<'_, AppState>,
) -> Result<CrawlProgress, String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    println!("\n{}", "=".repeat(80));
    println!("CRAWLER STARTED");
    if let Some(max) = max_pages {
        println!("Mode: Quick Start ({} pages)", max);
    } else {
        println!("Mode: Full Database (all pages)");
    }
    println!("{}", "=".repeat(80));
    
    // Create crawler
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let db_path = state.db_path.lock().unwrap().clone();
    
    // Crawl pages and save incrementally
    let mut total_games = 0;
    let mut current_page = 1u32;
    
    loop {
        // Check if we've reached max_pages
        if let Some(max) = max_pages {
            if current_page > max {
                break;
            }
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
                    println!(
                        "[OK] Page {}: Found {} games (Total: {})",
                        current_page, count, total_games
                    );
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
            // Insert or update repack
            db.conn.execute(
                "INSERT INTO repacks (title, genres_tags, company, languages, original_size, repack_size, url, date, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)
                 ON CONFLICT(url) DO UPDATE SET
                    title = excluded.title,
                    genres_tags = excluded.genres_tags,
                    company = excluded.company,
                    languages = excluded.languages,
                    original_size = excluded.original_size,
                    repack_size = excluded.repack_size,
                    date = excluded.date,
                    updated_at = CURRENT_TIMESTAMP",
                (&repack.title, &repack.genres_tags, &repack.company, &repack.languages, 
                 &repack.original_size, &repack.repack_size, &repack.url, &repack.date),
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
                
                // Filter out games that are older than or equal to our latest date
                let new_repacks: Vec<_> = repacks
                    .into_iter()
                    .filter(|r| {
                        if let Some(ref game_date) = r.date {
                            if let Some(ref latest) = latest_date {
                                game_date > latest
                            } else {
                                true
                            }
                        } else {
                            true
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

