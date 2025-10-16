use crate::constants::DATABASE_URL;
use crate::database::{AppSettings, Database};
use super::utils::AppState;
use std::fs;
use std::io::Write;
use tauri::State;

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

