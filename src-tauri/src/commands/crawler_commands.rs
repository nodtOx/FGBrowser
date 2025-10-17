use crate::crawler::{FitGirlCrawler, GameRepack, clean_game_title};
use crate::database::Database;
use super::utils::{AppState, parse_size_to_mb};
use super::database_service::{DatabaseService, SqliteDatabaseService};
use std::path::Path;
use std::sync::Arc;
use tauri::State;

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
    
    // Clone the service for use in crawler
    let db_service = Arc::clone(&state.db_service);
    
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
                if let Err(e) = save_repacks_to_db(&repacks, &db_service) {
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

#[tauri::command]
pub async fn update_database(
    state: State<'_, AppState>,
) -> Result<CrawlProgress, String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    
    // Get the latest game date
    let latest_date = state.db_service
        .get_latest_game_date()
        .map_err(|e| e.to_string())?;
    
    if latest_date.is_none() {
        return Ok(CrawlProgress {
            current_page: 0,
            total_games: 0,
            status: "No existing data".to_string(),
        });
    }
    
    // Create crawler
    let crawler = FitGirlCrawler::new().map_err(|e| e.to_string())?;
    let db_service = Arc::clone(&state.db_service);
    
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
                let new_repacks: Vec<_> = repacks
                    .into_iter()
                    .filter(|r| {
                        // Check if this URL already exists in database
                        match db_service.check_url_exists(&r.url) {
                            Ok(exists) => !exists,  // Include if doesn't exist
                            Err(_) => true,  // Include on error
                        }
                    })
                    .collect();
                
                if new_repacks.is_empty() {
                    break;
                }
                
                let count = new_repacks.len();
                
                // Save new games to database
                if let Err(e) = save_repacks_to_db(&new_repacks, &db_service) {
                    eprintln!("Error saving page {}: {}", current_page, e);
                } else {
                    total_new_games += count;
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
            break;
        }
    }
    
    let _duration = start_time.elapsed();
    
    Ok(CrawlProgress {
        current_page: current_page - 1,
        total_games: total_new_games,
        status: "Updated".to_string(),
    })
}

/// Save repacks to database using the DatabaseService
/// Uses with_connection for efficient bulk operations with transactions
pub fn save_repacks_to_db(repacks: &[GameRepack], db_service: &SqliteDatabaseService) -> anyhow::Result<()> {
    let mut saved_count = 0;
    
    // Use with_connection to get direct database access for bulk transaction
    db_service.with_connection(|db| {
        // Use transaction for bulk operations (100-1000x speedup)
        let tx = db.conn.unchecked_transaction()?;
        {
            // Prepare statements once, reuse many times
            let mut insert_repack_stmt = tx.prepare(
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
                    updated_at = CURRENT_TIMESTAMP"
            )?;
            
            let mut get_repack_id_stmt = tx.prepare("SELECT id FROM repacks WHERE url = ?1")?;
            let mut insert_magnet_stmt = tx.prepare(
                "INSERT INTO magnet_links (repack_id, source, magnet)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(repack_id, source) DO UPDATE SET magnet = excluded.magnet"
            )?;
            let mut insert_category_stmt = tx.prepare("INSERT OR IGNORE INTO categories (name) VALUES (?1)")?;
            let mut get_category_id_stmt = tx.prepare("SELECT id FROM categories WHERE name = ?1")?;
            let mut insert_game_category_stmt = tx.prepare("INSERT OR IGNORE INTO game_categories (repack_id, category_id) VALUES (?1, ?2)")?;
            
            for repack in repacks {
                let save_result = (|| -> anyhow::Result<()> {
                    let parsed_size = parse_size_to_mb(&repack.repack_size);
                    let clean_name = clean_game_title(&repack.title);
                    
                    insert_repack_stmt.execute(
                        (&repack.title, &clean_name, &repack.genres_tags, &repack.company, &repack.languages, 
                         &repack.original_size, &repack.repack_size, &parsed_size, &repack.url, &repack.date, &repack.image_url),
                    )?;
                    
                    let repack_id: i64 = get_repack_id_stmt.query_row([&repack.url], |row| row.get(0))?;
                    
                    for magnet in &repack.magnet_links {
                        insert_magnet_stmt.execute((repack_id, &magnet.source, &magnet.magnet))?;
                    }
                    
                    if let Some(genres_tags) = &repack.genres_tags {
                        if !genres_tags.trim().is_empty() {
                            let categories: Vec<String> = genres_tags
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            
                            for category_name in categories {
                                insert_category_stmt.execute([&category_name])?;
                                let category_id: i64 = get_category_id_stmt.query_row([&category_name], |row| row.get(0))?;
                                insert_game_category_stmt.execute([repack_id, category_id])?;
                            }
                        }
                    }
                    
                    Ok(())
                })();
                
                match save_result {
                    Ok(_) => saved_count += 1,
                    Err(e) => eprintln!("Failed to save repack '{}': {}", repack.title, e),
                }
            }
        }
        tx.commit()?;
        Ok(())
    })?;
    
    if saved_count > 0 {
        println!("Saved {}/{} repacks to database", saved_count, repacks.len());
        Database::clear_category_cache();
        println!("🧹 Category cache cleared after adding {} games", saved_count);
    }
    
    Ok(())
}

/// Legacy save function that takes a PathBuf
/// Kept for backward compatibility with CLI
pub fn save_repacks_to_db_legacy(repacks: &[GameRepack], db_path: &Path) -> anyhow::Result<()> {
    let db = Database::new(db_path.to_path_buf())?;
    
    // Process all repacks in a single transaction for 100-1000x speedup
    let mut saved_count = 0;
    
    // Use transaction for bulk operations
    let tx = db.conn.unchecked_transaction()?;
    {
        // Prepare statements once, reuse many times
        let mut insert_repack_stmt = tx.prepare(
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
                updated_at = CURRENT_TIMESTAMP"
        )?;
        
        let mut get_repack_id_stmt = tx.prepare("SELECT id FROM repacks WHERE url = ?1")?;
        let mut insert_magnet_stmt = tx.prepare(
            "INSERT INTO magnet_links (repack_id, source, magnet)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(repack_id, source) DO UPDATE SET magnet = excluded.magnet"
        )?;
        let mut insert_category_stmt = tx.prepare("INSERT OR IGNORE INTO categories (name) VALUES (?1)")?;
        let mut get_category_id_stmt = tx.prepare("SELECT id FROM categories WHERE name = ?1")?;
        let mut insert_game_category_stmt = tx.prepare("INSERT OR IGNORE INTO game_categories (repack_id, category_id) VALUES (?1, ?2)")?;
        
        for repack in repacks {
            // Use a closure to handle individual game saves
            let save_result = (|| -> anyhow::Result<()> {
                // Parse size from repack_size for the integer column
                let parsed_size = parse_size_to_mb(&repack.repack_size);
                
                // Generate clean name
                let clean_name = clean_game_title(&repack.title);
                
                // Insert or update repack
                insert_repack_stmt.execute(
                    (&repack.title, &clean_name, &repack.genres_tags, &repack.company, &repack.languages, 
                     &repack.original_size, &repack.repack_size, &parsed_size, &repack.url, &repack.date, &repack.image_url),
                )?;
                
                // Get repack_id
                let repack_id: i64 = get_repack_id_stmt.query_row([&repack.url], |row| row.get(0))?;
                
                // Insert magnet links
                for magnet in &repack.magnet_links {
                    insert_magnet_stmt.execute((repack_id, &magnet.source, &magnet.magnet))?;
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
                            insert_category_stmt.execute([&category_name])?;
                            
                            // Get category ID
                            let category_id: i64 = get_category_id_stmt.query_row([&category_name], |row| row.get(0))?;
                            
                            // Insert game-category relationship
                            insert_game_category_stmt.execute([repack_id, category_id])?;
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
    }
    tx.commit()?;
    
    if saved_count > 0 {
        println!("Saved {}/{} repacks to database", saved_count, repacks.len());
        // Clear category cache since new games were added
        Database::clear_category_cache();
        println!("🧹 Category cache cleared after adding {} games", saved_count);
    }
    
    Ok(())
}
