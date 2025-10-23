use crate::crawler::FitGirlCrawler;
use tauri::State;
use super::utils::AppState;
use super::database_service::DatabaseService;

#[derive(Debug, serde::Serialize, Clone)]
pub struct Screenshot {
    pub url: String,
    pub thumbnail_url: Option<String>,
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
/// First checks database, if not found or missing thumbnails, fetches from FitGirl page
#[tauri::command]
pub async fn fetch_game_media(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaResult, String> {
    // First, try to get from database
    let existing_media = get_game_media_from_db(game_id, &state)?;
    
    // Check if we have screenshots with thumbnails
    let has_thumbnails = existing_media.screenshots.iter().any(|s| s.thumbnail_url.is_some());
    
    // If we have screenshots with thumbnails and videos, return them
    if !existing_media.screenshots.is_empty() && has_thumbnails {
        return Ok(existing_media);
    }
    
    // If we have screenshots but no thumbnails, we need to refetch
    let need_refetch = !existing_media.screenshots.is_empty() && !has_thumbnails;
    
    // Otherwise, fetch from FitGirl page
    if need_refetch {
        println!("🔄 Refetching screenshots to populate thumbnails for game {}", game_id);
    } else {
        println!("📸 Fetching media for game {}", game_id);
    }
    
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
        // Save screenshots and videos to database with thumbnails
        save_media_to_db_with_thumbnails(game_id, &repack.screenshot_data, &repack.videos, &state)?;
        
        // Return the media
        Ok(MediaResult {
            screenshots: repack.screenshot_data.into_iter().map(|s| Screenshot { 
                url: s.full_url, 
                thumbnail_url: s.thumbnail_url 
            }).collect(),
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
            "SELECT screenshot_url, thumbnail_url FROM screenshots WHERE repack_id = ?1"
        )?;
        
        let screenshots: Vec<Screenshot> = stmt.query_map([game_id], |row| {
            Ok(Screenshot {
                url: row.get(0)?,
                thumbnail_url: row.get(1)?,
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

fn save_media_to_db_with_thumbnails(
    game_id: i64,
    screenshots: &[crate::crawler::riotpixels::ScreenshotData],
    videos: &[String],
    state: &AppState,
) -> Result<(), String> {
    state.db_service.with_connection(|db| {
        let tx = db.conn.unchecked_transaction()?;
        
        {
            // Insert or update screenshots with thumbnails
            let mut insert_screenshot_stmt = tx.prepare(
                "INSERT INTO screenshots (repack_id, screenshot_url, thumbnail_url)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(repack_id, screenshot_url) DO UPDATE SET thumbnail_url = excluded.thumbnail_url"
            )?;
            
            for screenshot in screenshots {
                insert_screenshot_stmt.execute((
                    game_id, 
                    &screenshot.full_url, 
                    screenshot.thumbnail_url.as_ref()
                ))?;
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
        
        let screenshots_with_thumbnails = screenshots.iter().filter(|s| s.thumbnail_url.is_some()).count();
        println!("💾 Saved {} screenshots ({} with thumbnails) and {} videos for game {}", 
                 screenshots.len(), screenshots_with_thumbnails, videos.len(), game_id);
        
        Ok(())
    }).map_err(|e| format!("Failed to save media: {}", e))
}

