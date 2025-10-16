use crate::crawler::FitGirlCrawler;
use crate::database::{PopularRepack, PopularRepackWithGame};
use super::utils::{AppState, is_popular_blacklisted};
use super::crawler_commands::save_repacks_to_db;
use super::database_service::DatabaseService;
use tauri::State;
use std::sync::Arc;

#[tauri::command]
pub async fn fetch_popular_repacks(
    period: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let popular_entries = crawler.fetch_popular_repacks(&period).await.map_err(|e| e.to_string())?;
    
    // Clear existing popular repacks for this period
    state.db_service.clear_popular_repacks(&period).map_err(|e| e.to_string())?;
    
    // Save new popular repacks with rank based on order (1, 2, 3...)
    let mut saved_count = 0;
    for (index, entry) in popular_entries.iter().enumerate() {
        let rank = (index + 1) as i32;
        match state.db_service.save_popular_repack(
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
    let _linked_count = state.db_service.update_popular_repack_links(Some(&period)).map_err(|e| e.to_string())?;
    
    Ok(saved_count)
}

#[tauri::command]
pub async fn parse_popular_repacks_from_file(
    file_path: String,
    period: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    println!("📄 Parsing popular repacks from file: {}", file_path);
    
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let popular_entries = crawler.parse_popular_repacks_from_file(&file_path).map_err(|e| e.to_string())?;
    
    // Clear existing popular repacks for this period
    state.db_service.clear_popular_repacks(&period).map_err(|e| e.to_string())?;
    
    // Save new popular repacks with rank based on order (1, 2, 3...)
    let mut saved_count = 0;
    for (index, entry) in popular_entries.iter().enumerate() {
        let rank = (index + 1) as i32;
        match state.db_service.save_popular_repack(
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
    let _linked_count = state.db_service.update_popular_repack_links(Some(&period)).map_err(|e| e.to_string())?;
    
    Ok(saved_count)
}

#[tauri::command]
pub async fn get_popular_repacks(
    period: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<PopularRepack>, String> {
    let mut repacks = state.db_service.get_popular_repacks(&period, limit).map_err(|e| e.to_string())?;
    
    // Apply blacklist filtering for all periods except award (which is curated)
    if period != "award" {
        repacks.retain(|repack| !is_popular_blacklisted(&repack.url));
    }
    
    Ok(repacks)
}

#[tauri::command]
pub async fn get_popular_repacks_with_games(
    period: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<PopularRepackWithGame>, String> {
    let mut repacks = state.db_service.get_popular_repacks_with_games(&period, limit).map_err(|e| e.to_string())?;
    
    // Apply blacklist filtering for all periods except award (which is curated)
    if period != "award" {
        repacks.retain(|repack| !is_popular_blacklisted(&repack.url));
    }
    
    Ok(repacks)
}

#[tauri::command]
pub async fn get_unseen_popular_count(
    period: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    // Get all popular repacks for the period (with a high limit to get all)
    let mut repacks = state.db_service.get_popular_repacks(&period, 500).map_err(|e| e.to_string())?;
    
    // Apply blacklist filtering for all periods except award
    if period != "award" {
        repacks.retain(|repack| !is_popular_blacklisted(&repack.url));
    }
    
    // Now count unseen ones
    let settings = state.db_service.get_settings().map_err(|e| e.to_string())?;
    let last_viewed = match period.as_str() {
        "week" => settings.popular_week_last_viewed.as_deref(),
        "today" => settings.popular_today_last_viewed.as_deref(),
        "month" => settings.popular_month_last_viewed.as_deref(),
        "year" => settings.popular_year_last_viewed.as_deref(),
        "award" => settings.popular_award_last_viewed.as_deref(),
        _ => None,
    };
    
    let unseen_count = match last_viewed {
        Some(timestamp) => {
            repacks.iter().filter(|repack| {
                if let Some(created_at) = &repack.created_at {
                    created_at.as_str() > timestamp
                } else {
                    false
                }
            }).count()
        }
        None => repacks.len(),
    };
    
    Ok(unseen_count as i64)
}

#[tauri::command]
pub async fn get_total_unseen_popular_count(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let settings = state.db_service.get_settings().map_err(|e| e.to_string())?;
    
    // Calculate unseen count for each period with blacklist filtering
    let periods = vec![
        ("week", settings.popular_week_last_viewed.as_deref()),
        ("today", settings.popular_today_last_viewed.as_deref()),
        ("month", settings.popular_month_last_viewed.as_deref()),
        ("year", settings.popular_year_last_viewed.as_deref()),
        ("award", settings.popular_award_last_viewed.as_deref()),
    ];
    
    let mut total_unseen = 0i64;
    
    for (period, last_viewed) in periods {
        let mut repacks = state.db_service.get_popular_repacks(period, 500).map_err(|e| e.to_string())?;
        
        // Apply blacklist filtering for all periods except award
        if period != "award" {
            repacks.retain(|repack| !is_popular_blacklisted(&repack.url));
        }
        
        // Count unseen ones
        let unseen_count = match last_viewed {
            Some(timestamp) => {
                repacks.iter().filter(|repack| {
                    if let Some(created_at) = &repack.created_at {
                        created_at.as_str() > timestamp
                    } else {
                        false
                    }
                }).count()
            }
            None => repacks.len(),
        };
        
        total_unseen += unseen_count as i64;
    }
    
    Ok(total_unseen)
}

#[tauri::command]
pub async fn mark_popular_as_viewed(
    period: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.db_service.get_settings().map_err(|e| e.to_string())?;
    
    // Get current timestamp in ISO 8601 format
    let now = chrono::Utc::now().to_rfc3339();
    
    // Update the appropriate timestamp
    match period.as_str() {
        "week" => settings.popular_week_last_viewed = Some(now),
        "today" => settings.popular_today_last_viewed = Some(now),
        "month" => settings.popular_month_last_viewed = Some(now),
        "year" => settings.popular_year_last_viewed = Some(now),
        "award" => settings.popular_award_last_viewed = Some(now),
        _ => return Err("Invalid period".to_string()),
    }
    
    state.db_service.save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_popular_repack_links(
    period: Option<String>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let count = state.db_service.update_popular_repack_links(period.as_deref()).map_err(|e| e.to_string())?;
    println!("🔗 Updated links for {} popular repacks", count);
    Ok(count)
}

#[tauri::command]
pub async fn crawl_popular_games(
    period: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    // Get popular repacks for the specified period
    let popular_repacks = state.db_service.get_popular_repacks(&period, 200).map_err(|e| e.to_string())?;
    
    if popular_repacks.is_empty() {
        return Err(format!("No popular repacks ({}) found. Fetch popular repacks first.", period));
    }
    
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let db_service = Arc::clone(&state.db_service);
    let mut crawled_count = 0;
    let mut _skipped_count = 0;
    
    for popular in &popular_repacks {
        // Skip if already linked to a game in database
        if popular.repack_id.is_some() {
            _skipped_count += 1;
            continue;
        }
        
        // Crawl the game page
        match crawler.crawl_single_game(&popular.url).await {
            Ok(Some(game_repack)) => {
                // Save to database
                if let Err(e) = save_repacks_to_db(&[game_repack], &*db_service) {
                    eprintln!("    ❌ Failed to save: {}", e);
                } else {
                    crawled_count += 1;
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
    let _linked = state.db_service.update_popular_repack_links(Some(&period)).map_err(|e| e.to_string())?;
    
    Ok(crawled_count)
}

#[tauri::command]
pub async fn crawl_single_popular_game(
    url: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    println!("🎮 Crawling single game: {}", url);
    
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let db_service = Arc::clone(&state.db_service);
    
    match crawler.crawl_single_game(&url).await {
        Ok(Some(game_repack)) => {
            // Save to database
            save_repacks_to_db(&[game_repack], &*db_service).map_err(|e| e.to_string())?;
            
            // Update links for all periods
            state.db_service.update_popular_repack_links(None).map_err(|e| e.to_string())?;
            
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
