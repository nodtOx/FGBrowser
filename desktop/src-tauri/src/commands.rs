use crate::database::{Database, Game, GameDetails, DatabaseStats};
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

