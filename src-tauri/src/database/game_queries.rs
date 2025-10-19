use rusqlite::Result;
use super::models::{Game, GameDetails, MagnetLink, Category, DatabaseStats};
use super::cache;

// SQL query helpers to reduce repetition
// Note: is_new field is calculated via CASE statement in queries
pub const GAME_SELECT_FIELDS: &str = "id, title, clean_name, genres_tags, company, languages, original_size, repack_size, size, url, date, image_url, \
    CASE WHEN created_at > COALESCE((SELECT json_extract(value, '$.games_last_seen_date') FROM settings WHERE key = 'app_settings'), '1970-01-01') \
    THEN 1 ELSE 0 END as is_new";
pub const GAME_SELECT_FIELDS_PREFIXED: &str = "r.id, r.title, r.clean_name, r.genres_tags, r.company, r.languages, r.original_size, r.repack_size, r.size, r.url, r.date, r.image_url, \
    CASE WHEN r.created_at > COALESCE((SELECT json_extract(value, '$.games_last_seen_date') FROM settings WHERE key = 'app_settings'), '1970-01-01') \
    THEN 1 ELSE 0 END as is_new";

// Helper function to map a row to a Game struct
// This eliminates repetitive mapping code across all queries
pub fn map_row_to_game(row: &rusqlite::Row) -> Result<Game> {
    Ok(Game {
        id: row.get(0)?,
        title: row.get(1)?,
        clean_name: row.get(2)?,
        genres_tags: row.get(3)?,
        company: row.get(4)?,
        languages: row.get(5)?,
        original_size: row.get(6)?,
        repack_size: row.get(7)?,
        size: row.get(8)?,
        url: row.get(9)?,
        date: row.get(10)?,
        image_url: row.get(11)?,
        is_new: row.get::<_, i32>(12)? == 1,
    })
}

pub struct GameQueries;

impl GameQueries {
    pub fn search_games(conn: &rusqlite::Connection, query: &str, limit: i32) -> Result<Vec<Game>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM repacks 
             WHERE (title LIKE ?1 OR clean_name LIKE ?1)
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)
             ORDER BY 
               CASE 
                 WHEN clean_name LIKE ?1 THEN 1 
                 WHEN title LIKE ?1 THEN 2 
                 ELSE 3 
               END,
               date DESC 
             LIMIT ?2",
            GAME_SELECT_FIELDS
        ))?;

        let games = stmt.query_map([&search_pattern, &limit.to_string()], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }

    pub fn get_all_games(conn: &rusqlite::Connection, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM repacks 
             WHERE EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)
             ORDER BY date DESC LIMIT ?1 OFFSET ?2",
            GAME_SELECT_FIELDS
        ))?;

        let games = stmt.query_map([&limit.to_string(), &offset.to_string()], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }

    pub fn get_game_details(conn: &rusqlite::Connection, game_id: i64) -> Result<GameDetails> {
        // Get game info
        let game: Game = conn.query_row(
            &format!("SELECT {} FROM repacks WHERE id = ?1", GAME_SELECT_FIELDS),
            [game_id],
            map_row_to_game,
        )?;

        // Get magnet links
        let mut stmt = conn.prepare(
            "SELECT id, repack_id, source, magnet FROM magnet_links WHERE repack_id = ?1",
        )?;

        let magnet_links = stmt
            .query_map([game_id], |row| {
                Ok(MagnetLink {
                    id: row.get(0)?,
                    repack_id: row.get(1)?,
                    source: row.get(2)?,
                    magnet: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        // Get categories
        let mut stmt = conn.prepare(
            "SELECT c.id, c.name 
             FROM categories c 
             JOIN game_categories gc ON c.id = gc.category_id 
             WHERE gc.repack_id = ?1
             ORDER BY c.name",
        )?;

        let categories = stmt
            .query_map([game_id], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(GameDetails { game, magnet_links, categories })
    }

    pub fn get_stats(conn: &rusqlite::Connection) -> Result<DatabaseStats> {
        let total_games: i64 = conn
            .query_row("SELECT COUNT(*) FROM repacks WHERE EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)", [], |row| row.get(0))?;

        let total_magnets: i64 =
            conn
                .query_row("SELECT COUNT(*) FROM magnet_links", [], |row| row.get(0))?;

        Ok(DatabaseStats {
            total_games,
            total_magnets,
        })
    }
    
    pub fn get_latest_game_date(conn: &rusqlite::Connection) -> Result<Option<String>> {
        match conn.query_row(
            "SELECT date FROM repacks WHERE date IS NOT NULL 
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)
             ORDER BY date DESC LIMIT 1",
            [],
            |row| row.get(0),
        ) {
            Ok(date) => Ok(Some(date)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    // Get games filtered by date range
    pub fn get_games_by_date_range(conn: &rusqlite::Connection, days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM repacks 
             WHERE date >= date('now', '-' || ? || ' days')
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)
             ORDER BY date DESC LIMIT ? OFFSET ?",
            GAME_SELECT_FIELDS
        ))?;

        let games = stmt.query_map([days_ago, limit, offset], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }
    
    // Get games filtered by size range (in MB)
    pub fn get_games_by_size_range(conn: &rusqlite::Connection, min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut query = format!("SELECT {} FROM repacks WHERE EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)", GAME_SELECT_FIELDS);
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(min) = min_size {
            query.push_str(" AND size >= ?");
            params.push(Box::new(min));
        }
        
        if let Some(max) = max_size {
            query.push_str(" AND size <= ?");
            params.push(Box::new(max));
        }
        
        query.push_str(" ORDER BY date DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        
        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let games = stmt.query_map(&param_refs[..], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }
    
    // Get games by categories AND size range (combined filters)
    pub fn get_games_by_categories_and_size(conn: &rusqlite::Connection, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> Result<Vec<Game>> {
        if category_ids.is_empty() {
            // No categories selected, just filter by size
            return Self::get_games_by_size_range(conn, min_size, max_size, limit, offset);
        }
        
        let placeholders = category_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let mut query = format!(
            "SELECT {} FROM repacks r
             WHERE r.id IN (
                 SELECT gc.repack_id 
                 FROM game_categories gc 
                 WHERE gc.category_id IN ({})
                 GROUP BY gc.repack_id 
                 HAVING COUNT(DISTINCT gc.category_id) = ?
             )
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = r.id)",
            GAME_SELECT_FIELDS_PREFIXED, placeholders
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        // Add category parameters
        for &id in category_ids {
            params.push(Box::new(id));
        }
        params.push(Box::new(category_ids.len() as i64));
        
        // Add size constraints
        if let Some(min) = min_size {
            query.push_str(" AND r.size >= ?");
            params.push(Box::new(min));
        }
        
        if let Some(max) = max_size {
            query.push_str(" AND r.size <= ?");
            params.push(Box::new(max));
        }
        
        query.push_str(" ORDER BY r.date DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        
        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let games = stmt.query_map(&param_refs[..], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }
    
    // Get games by categories, size AND time (triple combination)
    pub fn get_games_by_categories_size_and_time(conn: &rusqlite::Connection, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        if category_ids.is_empty() {
            // No categories selected, filter by size and time only
            return Self::get_games_by_size_and_time(conn, min_size, max_size, days_ago, limit, offset);
        }
        
        let placeholders = category_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let mut query = format!(
            "SELECT {} FROM repacks r
             WHERE r.id IN (
                 SELECT gc.repack_id 
                 FROM game_categories gc 
                 WHERE gc.category_id IN ({})
                 GROUP BY gc.repack_id 
                 HAVING COUNT(DISTINCT gc.category_id) = ?
             )
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = r.id)",
            GAME_SELECT_FIELDS_PREFIXED, placeholders
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        // Add category parameters
        for &id in category_ids {
            params.push(Box::new(id));
        }
        params.push(Box::new(category_ids.len() as i64));
        
        // Add size constraints
        if let Some(min) = min_size {
            query.push_str(" AND r.size >= ?");
            params.push(Box::new(min));
        }
        
        if let Some(max) = max_size {
            query.push_str(" AND r.size <= ?");
            params.push(Box::new(max));
        }
        
        // Add time constraint
        query.push_str(" AND r.date >= date('now', '-' || ? || ' days')");
        params.push(Box::new(days_ago));
        
        query.push_str(" ORDER BY r.date DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        
        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let games = stmt.query_map(&param_refs[..], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }
    
    // Get games by size AND time (dual combination)
    pub fn get_games_by_size_and_time(conn: &rusqlite::Connection, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut query = format!("SELECT {} FROM repacks WHERE EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)", GAME_SELECT_FIELDS);
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(min) = min_size {
            query.push_str(" AND size >= ?");
            params.push(Box::new(min));
        }
        
        if let Some(max) = max_size {
            query.push_str(" AND size <= ?");
            params.push(Box::new(max));
        }
        
        query.push_str(" AND date >= date('now', '-' || ? || ' days')");
        params.push(Box::new(days_ago));
        
        query.push_str(" ORDER BY date DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        
        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let games = stmt.query_map(&param_refs[..], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }
    
    // Get games by categories AND time (dual combination)
    pub fn get_games_by_categories_and_time(conn: &rusqlite::Connection, category_ids: &[i64], days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        if category_ids.is_empty() {
            return Self::get_games_by_date_range(conn, days_ago, limit, offset);
        }
        
        let placeholders = category_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT {} FROM repacks r
             WHERE r.id IN (
                 SELECT gc.repack_id 
                 FROM game_categories gc 
                 WHERE gc.category_id IN ({})
                 GROUP BY gc.repack_id 
                 HAVING COUNT(DISTINCT gc.category_id) = ?
             )
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = r.id)
             AND r.date >= date('now', '-' || ? || ' days')
             ORDER BY r.date DESC
             LIMIT ? OFFSET ?",
            GAME_SELECT_FIELDS_PREFIXED, placeholders
        );
        
        let mut stmt = conn.prepare(&query)?;
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for &id in category_ids {
            params.push(Box::new(id));
        }
        params.push(Box::new(category_ids.len() as i64));
        params.push(Box::new(days_ago));
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let games = stmt.query_map(&param_refs[..], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }
    
    pub fn get_games_by_category(conn: &rusqlite::Connection, category_id: i64, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM repacks r
             JOIN game_categories gc ON r.id = gc.repack_id
             WHERE gc.category_id = ?1
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = r.id)
             ORDER BY r.date DESC
             LIMIT ?2 OFFSET ?3",
            GAME_SELECT_FIELDS_PREFIXED
        ))?;

        let games = stmt.query_map([&category_id.to_string(), &limit.to_string(), &offset.to_string()], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(games)
    }

    pub fn get_games_by_multiple_categories(conn: &rusqlite::Connection, category_ids: &[i64], limit: i32, offset: i32) -> Result<Vec<Game>> {
        if category_ids.is_empty() {
            return Self::get_all_games(conn, limit, offset);
        }
        
        // Only cache queries with no offset and reasonable limits for efficiency
        let should_cache = offset == 0 && limit <= 1000;
        let cache_key = if should_cache {
            cache::get_cache_key(category_ids)
        } else {
            String::new()
        };
        
        // Check cache first
        if should_cache {
            if let Some(cached_games) = cache::get_from_cache(&cache_key) {
                println!("🚀 Cache HIT for categories: {:?}", category_ids);
                // Apply limit to cached results
                let end = std::cmp::min(limit as usize, cached_games.len());
                return Ok(cached_games.into_iter().take(end).collect());
            }
            println!("🔍 Cache MISS for categories: {:?}", category_ids);
        }
        
        // Not in cache or shouldn't cache, query database
        let placeholders = category_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        
        let query = format!(
            "SELECT {} FROM repacks r
             WHERE r.id IN (
                 SELECT gc.repack_id 
                 FROM game_categories gc 
                 WHERE gc.category_id IN ({})
                 GROUP BY gc.repack_id 
                 HAVING COUNT(DISTINCT gc.category_id) = ?
             )
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = r.id)
             ORDER BY r.date DESC
             LIMIT ? OFFSET ?",
            GAME_SELECT_FIELDS_PREFIXED, placeholders
        );
        
        let mut stmt = conn.prepare(&query)?;
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for &id in category_ids {
            params.push(Box::new(id));
        }
        params.push(Box::new(category_ids.len() as i64));
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let games = stmt
            .query_map(&param_refs[..], map_row_to_game)?
            .collect::<Result<Vec<_>>>()?;

        // Cache the result if appropriate
        if should_cache && !games.is_empty() {
            cache::add_to_cache(cache_key, games.clone());
        }

        Ok(games)
    }
}

