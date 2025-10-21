use crate::crawler::FitGirlCrawler;
use tauri::State;
use super::utils::AppState;
use super::database_service::DatabaseService;

#[derive(Debug, serde::Serialize, Clone)]
pub struct Screenshot {
    pub url: String,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct Video {
    pub url: String,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct MediaResult {
    pub screenshots: Vec<Screenshot>,
    pub videos: Vec<Video>,
}

/// Fetch screenshots and videos for a game (lazy loading)
/// First checks database, if not found, fetches from FitGirl page
#[tauri::command]
pub async fn fetch_game_media(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaResult, String> {
    // First, try to get from database
    let existing_media = get_game_media_from_db(game_id, &state)?;
    
    // If we have screenshots and videos, return them
    if !existing_media.screenshots.is_empty() || !existing_media.videos.is_empty() {
        return Ok(existing_media);
    }
    
    // Otherwise, fetch from FitGirl page
    println!("📸 Fetching media for game {}", game_id);
    
    // Get game URL from database
    let game_url = state.db_service.with_connection(|db| {
        let url: String = db.conn.query_row(
            "SELECT url FROM repacks WHERE id = ?1",
            [game_id],
            |row| row.get(0),
        )?;
        Ok(url)
    }).map_err(|e| format!("Failed to get game URL: {}", e))?;
    
    // Create crawler and fetch the page
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let repack = crawler.crawl_single_game(&game_url).await
        .map_err(|e| format!("Failed to crawl game: {}", e))?;
    
    if let Some(repack) = repack {
        // Save screenshots and videos to database
        save_media_to_db(game_id, &repack.screenshots, &repack.videos, &state)?;
        
        // Return the media
        Ok(MediaResult {
            screenshots: repack.screenshots.into_iter().map(|url| Screenshot { url }).collect(),
            videos: repack.videos.into_iter().map(|url| Video { url }).collect(),
        })
    } else {
        Ok(MediaResult {
            screenshots: vec![],
            videos: vec![],
        })
    }
}

/// Get screenshots and videos from database
#[tauri::command]
pub fn get_game_media(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaResult, String> {
    get_game_media_from_db(game_id, &state)
}

fn get_game_media_from_db(
    game_id: i64,
    state: &AppState,
) -> Result<MediaResult, String> {
    state.db_service.with_connection(|db| {
        // Get screenshots
        let mut stmt = db.conn.prepare(
            "SELECT screenshot_url FROM screenshots WHERE repack_id = ?1"
        )?;
        
        let screenshots: Vec<Screenshot> = stmt.query_map([game_id], |row| {
            Ok(Screenshot {
                url: row.get(0)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        // Get videos
        let mut stmt = db.conn.prepare(
            "SELECT video_url FROM videos WHERE repack_id = ?1"
        )?;
        
        let videos: Vec<Video> = stmt.query_map([game_id], |row| {
            Ok(Video {
                url: row.get(0)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(MediaResult { screenshots, videos })
    }).map_err(|e| format!("Failed to get game media: {}", e))
}

fn save_media_to_db(
    game_id: i64,
    screenshots: &[String],
    videos: &[String],
    state: &AppState,
) -> Result<(), String> {
    state.db_service.with_connection(|db| {
        let tx = db.conn.unchecked_transaction()?;
        
        {
            // Insert screenshots
            let mut insert_screenshot_stmt = tx.prepare(
                "INSERT INTO screenshots (repack_id, screenshot_url)
                 VALUES (?1, ?2)
                 ON CONFLICT(repack_id, screenshot_url) DO NOTHING"
            )?;
            
            for screenshot_url in screenshots {
                insert_screenshot_stmt.execute((game_id, screenshot_url))?;
            }
            
            // Insert videos
            let mut insert_video_stmt = tx.prepare(
                "INSERT INTO videos (repack_id, video_url)
                 VALUES (?1, ?2)
                 ON CONFLICT(repack_id, video_url) DO NOTHING"
            )?;
            
            for video_url in videos {
                insert_video_stmt.execute((game_id, video_url))?;
            }
        }
        
        tx.commit()?;
        
        println!("💾 Saved {} screenshots and {} videos for game {}", 
                 screenshots.len(), videos.len(), game_id);
        
        Ok(())
    }).map_err(|e| format!("Failed to save media: {}", e))
}

