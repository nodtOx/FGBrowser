use crate::crawler::{FitGirlCrawler, GameRepack};
use crate::database::{AppSettings, Database, Game, GameDetails, DatabaseStats};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use arboard::Clipboard;

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
    // Create crawler
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    
    // Crawl pages
    let repacks = crawler
        .crawl_multiple_pages(1, max_pages)
        .await
        .map_err(|e| e.to_string())?;
    
    // Save to database
    let db_path = state.db_path.lock().unwrap().clone();
    save_repacks_to_db(&repacks, &db_path).map_err(|e| e.to_string())?;
    
    Ok(CrawlProgress {
        current_page: max_pages.unwrap_or(0),
        total_games: repacks.len(),
        status: "Completed".to_string(),
    })
}

fn save_repacks_to_db(repacks: &[GameRepack], db_path: &PathBuf) -> anyhow::Result<()> {
    let db = Database::new(db_path.clone())?;
    
    for repack in repacks {
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
    }
    
    println!("Saved {} repacks to database", repacks.len());
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

