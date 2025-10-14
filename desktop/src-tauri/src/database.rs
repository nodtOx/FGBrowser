use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::LazyLock;

// Global query cache for category filtering
static CATEGORY_QUERY_CACHE: LazyLock<Mutex<HashMap<String, Vec<Game>>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MAX_CACHE_ENTRIES: usize = 100;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub genres_tags: Option<String>,
    pub company: Option<String>,
    pub languages: Option<String>,
    pub original_size: Option<String>,
    pub repack_size: Option<String>,
    pub size: Option<i64>, // Size in MB (parsed from repack_size)
    pub url: String,
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MagnetLink {
    pub id: i64,
    pub repack_id: i64,
    pub source: String,
    pub magnet: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameDetails {
    #[serde(flatten)]
    pub game: Game,
    pub magnet_links: Vec<MagnetLink>,
    pub categories: Vec<Category>,
}

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }
    
    
    pub fn init_tables(&self) -> Result<()> {
        // Create repacks table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS repacks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                genres_tags TEXT,
                company TEXT,
                languages TEXT,
                original_size TEXT,
                repack_size TEXT,
                size INTEGER,
                url TEXT UNIQUE,
                date TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // Create magnet_links table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS magnet_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repack_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                magnet TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repack_id) REFERENCES repacks (id) ON DELETE CASCADE,
                UNIQUE(repack_id, source)
            )",
            [],
        )?;
        
        // Create categories table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // Create game_categories junction table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS game_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repack_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repack_id) REFERENCES repacks (id) ON DELETE CASCADE,
                FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE CASCADE,
                UNIQUE(repack_id, category_id)
            )",
            [],
        )?;
        
        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_repacks_title ON repacks(title)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_repacks_date ON repacks(date)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_magnet_links_repack_id ON magnet_links(repack_id)",
            [],
        )?;
        
        // Create indexes for new tables
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_categories_name ON categories(name)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_game_categories_repack_id ON game_categories(repack_id)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_game_categories_category_id ON game_categories(category_id)",
            [],
        )?;
        
        // Run migration to convert genres_tags to m2m structure
        self.migrate_categories_data()?;
        
        Ok(())
    }
    
    pub fn migrate_categories_data(&self) -> Result<()> {
        // Check if migration is needed (if categories table is empty but repacks have genres_tags)
        let categories_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM categories",
            [],
            |row| row.get(0),
        )?;
        
        let repacks_with_genres: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM repacks WHERE genres_tags IS NOT NULL AND genres_tags != ''",
            [],
            |row| row.get(0),
        )?;
        
        // If categories table is empty but we have repacks with genres, run migration
        if categories_count == 0 && repacks_with_genres > 0 {
            println!("🔄 Migrating genre/tag data to new category structure...");
            
            // Get all unique categories from existing data
            let mut stmt = self.conn.prepare(
                "SELECT id, genres_tags FROM repacks WHERE genres_tags IS NOT NULL AND genres_tags != ''"
            )?;
            
            let mut category_map = std::collections::HashMap::<String, i64>::new();
            
            let repack_rows = stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            
            for repack_row in repack_rows {
                let (repack_id, genres_tags) = repack_row?;
                
                // Split by comma and clean up
                let categories: Vec<String> = genres_tags
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                for category_name in categories {
                    // Insert category if not exists and get ID
                    let category_id = if let Some(&existing_id) = category_map.get(&category_name) {
                        existing_id
                    } else {
                        // Insert new category
                        self.conn.execute(
                            "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
                            [&category_name],
                        )?;
                        
                        // Get the category ID
                        let category_id: i64 = self.conn.query_row(
                            "SELECT id FROM categories WHERE name = ?1",
                            [&category_name],
                            |row| row.get(0),
                        )?;
                        
                        category_map.insert(category_name.clone(), category_id);
                        category_id
                    };
                    
                    // Insert game-category relationship
                    self.conn.execute(
                        "INSERT OR IGNORE INTO game_categories (repack_id, category_id) VALUES (?1, ?2)",
                        [repack_id, category_id],
                    )?;
                }
            }
            
            println!("✅ Migration completed! Created {} categories", category_map.len());
        }
        
        Ok(())
    }

    pub fn search_games(&self, query: &str, limit: i32) -> Result<Vec<Game>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, size, url, date 
             FROM repacks 
             WHERE title LIKE ?1 
             ORDER BY date DESC 
             LIMIT ?2"
        )?;

        let games = stmt
            .query_map(&[&search_pattern, &limit.to_string()], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    size: row.get(7)?,
                    url: row.get(8)?,
                    date: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_all_games(&self, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, size, url, date 
             FROM repacks 
             ORDER BY date DESC 
             LIMIT ?1 OFFSET ?2"
        )?;

        let games = stmt
            .query_map(&[&limit.to_string(), &offset.to_string()], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    size: row.get(7)?,
                    url: row.get(8)?,
                    date: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_game_details(&self, game_id: i64) -> Result<GameDetails> {
        // Get game info
        let game: Game = self.conn.query_row(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, size, url, date 
             FROM repacks WHERE id = ?1",
            [game_id],
            |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    size: row.get(7)?,
                    url: row.get(8)?,
                    date: row.get(9)?,
                })
            },
        )?;

        // Get magnet links
        let mut stmt = self.conn.prepare(
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
        let mut stmt = self.conn.prepare(
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

    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let total_games: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM repacks", [], |row| row.get(0))?;

        let total_magnets: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM magnet_links", [], |row| row.get(0))?;

        Ok(DatabaseStats {
            total_games,
            total_magnets,
        })
    }
    
    pub fn get_latest_game_date(&self) -> Result<Option<String>> {
        match self.conn.query_row(
            "SELECT date FROM repacks WHERE date IS NOT NULL ORDER BY date DESC LIMIT 1",
            [],
            |row| row.get(0),
        ) {
            Ok(date) => Ok(Some(date)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    pub fn get_categories_with_counts(&self) -> Result<Vec<CategoryWithCount>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.name, COUNT(gc.repack_id) as game_count
             FROM categories c
             LEFT JOIN game_categories gc ON c.id = gc.category_id
             GROUP BY c.id, c.name
             HAVING game_count > 0
             ORDER BY game_count DESC, c.name ASC"
        )?;

        let categories = stmt
            .query_map([], |row| {
                Ok(CategoryWithCount {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    game_count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(categories)
    }

    // Get categories that appear in games filtered by selected categories
    pub fn get_categories_for_filtered_games(&self, selected_category_ids: &[i64]) -> Result<Vec<CategoryWithCount>> {
        if selected_category_ids.is_empty() {
            // If no categories selected, return all categories
            return self.get_categories_with_counts();
        }

        let placeholders = selected_category_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        
        // Find games that have ALL selected categories (same logic as game filtering)
        // Then count categories within those filtered games
        let query = format!(
            "SELECT c.id, c.name, COUNT(DISTINCT filtered_games.repack_id) as game_count
             FROM categories c
             JOIN game_categories gc ON c.id = gc.category_id
             JOIN (
                 SELECT gc.repack_id 
                 FROM game_categories gc 
                 WHERE gc.category_id IN ({})
                 GROUP BY gc.repack_id 
                 HAVING COUNT(DISTINCT gc.category_id) = ?
             ) filtered_games ON gc.repack_id = filtered_games.repack_id
             GROUP BY c.id, c.name
             HAVING game_count > 0
             ORDER BY game_count DESC, c.name ASC",
            placeholders
        );

        let mut stmt = self.conn.prepare(&query)?;
        
        // Build parameters: selected category IDs + count of selected categories
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for &id in selected_category_ids {
            params.push(Box::new(id));
        }
        params.push(Box::new(selected_category_ids.len() as i64));
        
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let categories = stmt
            .query_map(&param_refs[..], |row| {
                Ok(CategoryWithCount {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    game_count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(categories)
    }
    
    // Get games filtered by date range
    pub fn get_games_by_date_range(&self, days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, size, url, date 
             FROM repacks 
             WHERE date >= date('now', '-' || ? || ' days')
             ORDER BY date DESC
             LIMIT ? OFFSET ?"
        )?;

        let games = stmt
            .query_map([days_ago, limit, offset], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    size: row.get(7)?,
                    url: row.get(8)?,
                    date: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }
    
    pub fn get_games_by_category(&self, category_id: i64, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.title, r.genres_tags, r.company, r.languages, r.original_size, r.repack_size, r.size, r.url, r.date 
             FROM repacks r
             JOIN game_categories gc ON r.id = gc.repack_id
             WHERE gc.category_id = ?1
             ORDER BY r.date DESC
             LIMIT ?2 OFFSET ?3"
        )?;

        let games = stmt
            .query_map(&[&category_id.to_string(), &limit.to_string(), &offset.to_string()], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    size: row.get(7)?,
                    url: row.get(8)?,
                    date: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }
    
    // Cache management functions
    fn get_cache_key(category_ids: &[i64]) -> String {
        let mut sorted_ids = category_ids.to_vec();
        sorted_ids.sort();
        sorted_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")
    }
    
    fn get_from_cache(cache_key: &str) -> Option<Vec<Game>> {
        let cache = CATEGORY_QUERY_CACHE.lock().ok()?;
        cache.get(cache_key).cloned()
    }
    
    fn add_to_cache(cache_key: String, games: Vec<Game>) {
        if let Ok(mut cache) = CATEGORY_QUERY_CACHE.lock() {
            // Remove oldest entries if cache is full
            if cache.len() >= MAX_CACHE_ENTRIES {
                let keys_to_remove: Vec<_> = cache.keys().take(cache.len() - MAX_CACHE_ENTRIES + 1).cloned().collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
            cache.insert(cache_key, games);
        }
    }
    
    pub fn clear_category_cache() {
        if let Ok(mut cache) = CATEGORY_QUERY_CACHE.lock() {
            cache.clear();
        }
    }

    pub fn get_games_by_multiple_categories(&self, category_ids: &[i64], limit: i32, offset: i32) -> Result<Vec<Game>> {
        if category_ids.is_empty() {
            return self.get_all_games(limit, offset);
        }
        
        // Only cache queries with no offset and reasonable limits for efficiency
        let should_cache = offset == 0 && limit <= 1000;
        let cache_key = if should_cache {
            Self::get_cache_key(category_ids)
        } else {
            String::new()
        };
        
        // Check cache first
        if should_cache {
            if let Some(cached_games) = Self::get_from_cache(&cache_key) {
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
            "SELECT r.id, r.title, r.genres_tags, r.company, r.languages, r.original_size, r.repack_size, r.size, r.url, r.date 
             FROM repacks r
             WHERE r.id IN (
                 SELECT gc.repack_id 
                 FROM game_categories gc 
                 WHERE gc.category_id IN ({})
                 GROUP BY gc.repack_id 
                 HAVING COUNT(DISTINCT gc.category_id) = ?
             )
             ORDER BY r.date DESC
             LIMIT ? OFFSET ?",
            placeholders
        );
        
        let mut stmt = self.conn.prepare(&query)?;
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for &id in category_ids {
            params.push(Box::new(id));
        }
        params.push(Box::new(category_ids.len() as i64));
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let games = stmt
            .query_map(&param_refs[..], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    size: row.get(7)?,
                    url: row.get(8)?,
                    date: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        // Cache the result if appropriate
        if should_cache && !games.is_empty() {
            Self::add_to_cache(cache_key, games.clone());
        }

        Ok(games)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_games: i64,
    pub total_magnets: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryWithCount {
    pub id: i64,
    pub name: String,
    pub game_count: i64,
}

// Settings management
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    // General
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub notifications: bool,

    // Download
    pub download_path: String,
    pub max_simultaneous_downloads: i32,
    pub auto_start_downloads: bool,
    pub seed_after_complete: bool,
    pub seed_ratio: f64,

    // Network
    pub max_download_speed: i32,
    pub max_upload_speed: i32,
    pub port: i32,
    pub use_upnp: bool,
    pub use_dht: bool,

    // Appearance
    pub font_size: i32,
    pub compact_mode: bool,
    pub show_thumbnails: bool,
    pub animations_enabled: bool,

    // Database
    pub db_path: String,
    pub auto_refresh: bool,
    pub refresh_interval: i32,
}

impl Database {
    pub fn init_settings_table(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<AppSettings> {
        // Ensure settings table exists
        let _ = self.init_settings_table();

        // Try to get settings from database
        match self.conn.query_row(
            "SELECT value FROM settings WHERE key = 'app_settings'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(json_str) => {
                // Parse JSON
                serde_json::from_str(&json_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Return default settings if not found
                Ok(AppSettings::default())
            }
            Err(e) => Err(e),
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        // Ensure settings table exists
        self.init_settings_table()?;

        // Serialize to JSON
        let json_str = serde_json::to_string(settings)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Insert or replace
        self.conn.execute(
            "INSERT INTO settings (key, value, updated_at) 
             VALUES ('app_settings', ?1, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET 
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP",
            [&json_str],
        )?;

        Ok(())
    }
}
