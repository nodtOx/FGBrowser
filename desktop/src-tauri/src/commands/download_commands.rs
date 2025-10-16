use crate::database::{Database, Download};
use super::utils::{AppState, extract_info_hash};
use std::fs;
use tauri::State;

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

