use rusqlite::{Connection, Result};
use std::path::PathBuf;

// Re-export all public modules
pub mod models;
pub mod migrations;
pub mod game_queries;
pub mod category_queries;
pub mod popular_queries;
pub mod downloads_queries;
pub mod settings;
pub mod cache;

// Re-export commonly used types for convenience
pub use models::*;
pub use game_queries::GameQueries;
pub use downloads_queries::DownloadQueries;
pub use category_queries::CategoryQueries;
pub use popular_queries::PopularQueries;
pub use settings::SettingsQueries;

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
                clean_name TEXT,
                genres_tags TEXT,
                company TEXT,
                languages TEXT,
                original_size TEXT,
                repack_size TEXT,
                size INTEGER,
                url TEXT UNIQUE,
                date TEXT,
                image_url TEXT,
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
        
        // Create popular_repacks table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS popular_repacks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                image_url TEXT,
                rank INTEGER NOT NULL,
                period TEXT NOT NULL DEFAULT 'month',
                repack_id INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repack_id) REFERENCES repacks (id) ON DELETE SET NULL,
                UNIQUE(url, period)
            )",
            [],
        )?;
        
        // Create downloads table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repack_id INTEGER NOT NULL,
                game_title TEXT NOT NULL,
                magnet_link TEXT NOT NULL,
                info_hash TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL DEFAULT 'queued',
                save_path TEXT NOT NULL,
                total_size INTEGER DEFAULT 0,
                downloaded_bytes INTEGER DEFAULT 0,
                uploaded_bytes INTEGER DEFAULT 0,
                download_speed INTEGER DEFAULT 0,
                upload_speed INTEGER DEFAULT 0,
                progress REAL DEFAULT 0.0,
                peers INTEGER DEFAULT 0,
                seeds INTEGER DEFAULT 0,
                eta_seconds INTEGER,
                error_message TEXT,
                started_at TIMESTAMP,
                completed_at TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repack_id) REFERENCES repacks (id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_repacks_title ON repacks(title)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_repacks_clean_name ON repacks(clean_name)",
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
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_popular_repacks_period_rank ON popular_repacks(period, rank)",
            [],
        )?;
        
        // Note: url_period index will be created by migration after normalization
        // self.conn.execute(
        //     "CREATE INDEX IF NOT EXISTS idx_popular_repacks_url_period ON popular_repacks(url, period)",
        //     [],
        // )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_repack_id ON downloads(repack_id)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_info_hash ON downloads(info_hash)",
            [],
        )?;
        
        // Run migrations
        migrations::migrate_categories_data(&self.conn)?;
        migrations::migrate_popular_repacks_period(&self.conn)?;
        migrations::migrate_repacks_image_url(&self.conn)?;
        migrations::migrate_repacks_clean_name(&self.conn)?;
        migrations::populate_clean_names(&self.conn)?;
        migrations::migrate_normalize_popular_repacks(&self.conn)?;
        
        Ok(())
    }

    // Game query methods - delegate to GameQueries
    pub fn search_games(&self, query: &str, limit: i32) -> Result<Vec<Game>> {
        GameQueries::search_games(&self.conn, query, limit)
    }

    pub fn get_all_games(&self, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_all_games(&self.conn, limit, offset)
    }

    pub fn get_game_details(&self, game_id: i64) -> Result<GameDetails> {
        GameQueries::get_game_details(&self.conn, game_id)
    }

    pub fn get_stats(&self) -> Result<DatabaseStats> {
        GameQueries::get_stats(&self.conn)
    }
    
    pub fn get_latest_game_date(&self) -> Result<Option<String>> {
        GameQueries::get_latest_game_date(&self.conn)
    }
    
    pub fn get_games_by_date_range(&self, days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_date_range(&self.conn, days_ago, limit, offset)
    }
    
    pub fn get_games_by_size_range(&self, min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_size_range(&self.conn, min_size, max_size, limit, offset)
    }
    
    pub fn get_games_by_categories_and_size(&self, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_categories_and_size(&self.conn, category_ids, min_size, max_size, limit, offset)
    }
    
    pub fn get_games_by_categories_size_and_time(&self, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_categories_size_and_time(&self.conn, category_ids, min_size, max_size, days_ago, limit, offset)
    }
    
    pub fn get_games_by_size_and_time(&self, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_size_and_time(&self.conn, min_size, max_size, days_ago, limit, offset)
    }
    
    pub fn get_games_by_categories_and_time(&self, category_ids: &[i64], days_ago: i32, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_categories_and_time(&self.conn, category_ids, days_ago, limit, offset)
    }
    
    pub fn get_games_by_category(&self, category_id: i64, limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_category(&self.conn, category_id, limit, offset)
    }

    pub fn get_games_by_multiple_categories(&self, category_ids: &[i64], limit: i32, offset: i32) -> Result<Vec<Game>> {
        GameQueries::get_games_by_multiple_categories(&self.conn, category_ids, limit, offset)
    }

    // Category query methods - delegate to CategoryQueries
    pub fn get_categories_with_counts(&self) -> Result<Vec<CategoryWithCount>> {
        CategoryQueries::get_categories_with_counts(&self.conn)
    }

    pub fn get_categories_for_filtered_games(&self, selected_category_ids: &[i64]) -> Result<Vec<CategoryWithCount>> {
        CategoryQueries::get_categories_for_filtered_games(&self.conn, selected_category_ids)
    }
    
    pub fn get_categories_for_time_filtered_games(&self, days_ago: i32) -> Result<Vec<CategoryWithCount>> {
        CategoryQueries::get_categories_for_time_filtered_games(&self.conn, days_ago)
    }
    
    pub fn get_categories_for_size_filtered_games(&self, min_size: Option<i64>, max_size: Option<i64>) -> Result<Vec<CategoryWithCount>> {
        CategoryQueries::get_categories_for_size_filtered_games(&self.conn, min_size, max_size)
    }
    
    pub fn get_categories_for_size_and_time_filtered_games(&self, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32) -> Result<Vec<CategoryWithCount>> {
        CategoryQueries::get_categories_for_size_and_time_filtered_games(&self.conn, min_size, max_size, days_ago)
    }

    pub fn get_categories_for_search(&self, search_query: &str) -> Result<Vec<CategoryWithCount>> {
        CategoryQueries::get_categories_for_search(&self.conn, search_query)
    }

    // Popular repack methods - delegate to PopularQueries
    pub fn save_popular_repack(&self, url: &str, title: &str, image_url: Option<&str>, rank: i32, period: &str) -> Result<i64> {
        PopularQueries::save_popular_repack(&self.conn, url, title, image_url, rank, period)
    }
    
    pub fn get_popular_repacks(&self, period: &str, limit: i32) -> Result<Vec<PopularRepack>> {
        PopularQueries::get_popular_repacks(&self.conn, period, limit)
    }
    
    pub fn get_popular_repacks_with_games(&self, period: &str, limit: i32) -> Result<Vec<PopularRepackWithGame>> {
        PopularQueries::get_popular_repacks_with_games(&self.conn, period, limit)
    }
    
    pub fn clear_popular_repacks(&self, period: &str) -> Result<()> {
        PopularQueries::clear_popular_repacks(&self.conn, period)
    }
    
    pub fn update_popular_repack_links(&self, period: Option<&str>) -> Result<usize> {
        PopularQueries::update_popular_repack_links(&self.conn, period)
    }

    // Settings methods - delegate to SettingsQueries
    pub fn get_settings(&self) -> Result<AppSettings> {
        SettingsQueries::get_settings(&self.conn)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        SettingsQueries::save_settings(&self.conn, settings)
    }

    // Download methods - delegate to DownloadQueries
    pub fn get_all_downloads(&self) -> Result<Vec<Download>> {
        DownloadQueries::get_all_downloads(&self.conn)
    }

    pub fn get_download_by_info_hash(&self, info_hash: &str) -> Result<Option<Download>> {
        DownloadQueries::get_download_by_info_hash(&self.conn, info_hash)
    }

    pub fn create_download(&self, repack_id: i64, game_title: &str, magnet_link: &str, info_hash: &str, save_path: &str) -> Result<i64> {
        DownloadQueries::create_download(&self.conn, repack_id, game_title, magnet_link, info_hash, save_path)
    }

    pub fn update_download_status(&self, info_hash: &str, status: &str, error_message: Option<&str>) -> Result<()> {
        DownloadQueries::update_download_status(&self.conn, info_hash, status, error_message)
    }

    pub fn update_download_progress(
        &self,
        info_hash: &str,
        total_size: i64,
        downloaded_bytes: i64,
        uploaded_bytes: i64,
        download_speed: i64,
        upload_speed: i64,
        progress: f64,
        peers: i32,
        seeds: i32,
        eta_seconds: Option<i64>,
    ) -> Result<()> {
        DownloadQueries::update_download_progress(
            &self.conn,
            info_hash,
            total_size,
            downloaded_bytes,
            uploaded_bytes,
            download_speed,
            upload_speed,
            progress,
            peers,
            seeds,
            eta_seconds,
        )
    }

    pub fn delete_download(&self, info_hash: &str) -> Result<()> {
        DownloadQueries::delete_download(&self.conn, info_hash)
    }

    // Cache management - delegate to cache module
    pub fn clear_category_cache() {
        cache::clear_category_cache();
    }
}

