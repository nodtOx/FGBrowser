use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub genres_tags: Option<String>,
    pub company: Option<String>,
    pub languages: Option<String>,
    pub original_size: Option<String>,
    pub repack_size: Option<String>,
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
pub struct GameDetails {
    #[serde(flatten)]
    pub game: Game,
    pub magnet_links: Vec<MagnetLink>,
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
        
        Ok(())
    }

    pub fn search_games(&self, query: &str, limit: i32) -> Result<Vec<Game>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, url, date 
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
                    url: row.get(7)?,
                    date: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_all_games(&self, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, url, date 
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
                    url: row.get(7)?,
                    date: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_game_details(&self, game_id: i64) -> Result<GameDetails> {
        // Get game info
        let game: Game = self.conn.query_row(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, url, date 
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
                    url: row.get(7)?,
                    date: row.get(8)?,
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

        Ok(GameDetails { game, magnet_links })
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_games: i64,
    pub total_magnets: i64,
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
