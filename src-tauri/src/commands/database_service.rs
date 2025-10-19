use crate::database::{
    AppSettings, CategoryWithCount, Database, DatabaseStats, Download, 
    Game, GameDetails, PopularRepack, PopularRepackWithGame,
};
use rusqlite::Result as SqliteResult;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Database service trait - abstraction for database operations
/// This follows the Dependency Inversion Principle
pub trait DatabaseService: Send + Sync {
    fn search_games(&self, query: &str, limit: i32) -> SqliteResult<Vec<Game>>;
    fn get_all_games(&self, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_game_details(&self, game_id: i64) -> SqliteResult<GameDetails>;
    fn get_stats(&self) -> SqliteResult<DatabaseStats>;
    fn get_categories_with_counts(&self) -> SqliteResult<Vec<CategoryWithCount>>;
    fn get_categories_for_filtered_games(&self, selected_category_ids: &[i64]) -> SqliteResult<Vec<CategoryWithCount>>;
    fn get_categories_for_time_filtered_games(&self, days_ago: i32) -> SqliteResult<Vec<CategoryWithCount>>;
    fn get_categories_for_size_filtered_games(&self, min_size: Option<i64>, max_size: Option<i64>) -> SqliteResult<Vec<CategoryWithCount>>;
    fn get_categories_for_size_and_time_filtered_games(&self, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32) -> SqliteResult<Vec<CategoryWithCount>>;
    fn get_categories_for_search(&self, search_query: &str) -> SqliteResult<Vec<CategoryWithCount>>;
    fn get_games_by_date_range(&self, days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_games_by_size_range(&self, min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_games_by_categories_and_size(&self, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_games_by_categories_and_time(&self, category_ids: &[i64], days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_games_by_size_and_time(&self, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_games_by_categories_size_and_time(&self, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_games_by_category(&self, category_id: i64, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_games_by_multiple_categories(&self, category_ids: &[i64], limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    fn get_latest_game_date(&self) -> SqliteResult<Option<String>>;
    fn get_settings(&self) -> SqliteResult<AppSettings>;
    fn save_settings(&self, settings: &AppSettings) -> SqliteResult<()>;
    fn get_popular_repacks(&self, period: &str, limit: i32) -> SqliteResult<Vec<PopularRepack>>;
    fn get_popular_repacks_with_games(&self, period: &str, limit: i32) -> SqliteResult<Vec<PopularRepackWithGame>>;
    fn save_popular_repack(&self, url: &str, title: &str, image_url: Option<&str>, rank: i32, period: &str) -> SqliteResult<i64>;
    fn clear_popular_repacks(&self, period: &str) -> SqliteResult<()>;
    fn update_popular_repack_links(&self, period: Option<&str>) -> SqliteResult<usize>;
    fn get_all_downloads(&self) -> SqliteResult<Vec<Download>>;
    fn get_download_by_info_hash(&self, info_hash: &str) -> SqliteResult<Option<Download>>;
    fn create_download(&self, repack_id: i64, game_title: &str, magnet_link: &str, info_hash: &str, save_path: &str) -> SqliteResult<i64>;
    fn update_download_status(&self, info_hash: &str, status: &str, error_message: Option<&str>) -> SqliteResult<()>;
    fn delete_download(&self, info_hash: &str) -> SqliteResult<()>;
    fn check_table_exists(&self, table_name: &str) -> SqliteResult<bool>;
    fn check_url_exists(&self, url: &str) -> SqliteResult<bool>;
    fn get_new_games_count(&self) -> SqliteResult<i64>;
    
    /// Get direct access to the database for bulk operations
    /// This is a pragmatic compromise for operations that need transaction control
    fn with_connection<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&Database) -> SqliteResult<R>;
}

/// Concrete implementation using SQLite Database
/// This implements the DatabaseService trait
pub struct SqliteDatabaseService {
    db: Arc<Mutex<Database>>,
}

impl SqliteDatabaseService {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let db = Database::new(db_path)?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
    
    /// Helper to execute operation with database lock
    fn with_db<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&Database) -> SqliteResult<R>,
    {
        let db = self.db.lock().unwrap();
        f(&db)
    }
}

impl DatabaseService for SqliteDatabaseService {
    fn search_games(&self, query: &str, limit: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.search_games(query, limit))
    }
    
    fn get_all_games(&self, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_all_games(limit, offset))
    }
    
    fn get_game_details(&self, game_id: i64) -> SqliteResult<GameDetails> {
        self.with_db(|db| db.get_game_details(game_id))
    }
    
    fn get_stats(&self) -> SqliteResult<DatabaseStats> {
        self.with_db(|db| db.get_stats())
    }
    
    fn get_categories_with_counts(&self) -> SqliteResult<Vec<CategoryWithCount>> {
        self.with_db(|db| db.get_categories_with_counts())
    }
    
    fn get_categories_for_filtered_games(&self, selected_category_ids: &[i64]) -> SqliteResult<Vec<CategoryWithCount>> {
        self.with_db(|db| db.get_categories_for_filtered_games(selected_category_ids))
    }
    
    fn get_categories_for_time_filtered_games(&self, days_ago: i32) -> SqliteResult<Vec<CategoryWithCount>> {
        self.with_db(|db| db.get_categories_for_time_filtered_games(days_ago))
    }
    
    fn get_categories_for_size_filtered_games(&self, min_size: Option<i64>, max_size: Option<i64>) -> SqliteResult<Vec<CategoryWithCount>> {
        self.with_db(|db| db.get_categories_for_size_filtered_games(min_size, max_size))
    }
    
    fn get_categories_for_size_and_time_filtered_games(&self, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32) -> SqliteResult<Vec<CategoryWithCount>> {
        self.with_db(|db| db.get_categories_for_size_and_time_filtered_games(min_size, max_size, days_ago))
    }
    
    fn get_categories_for_search(&self, search_query: &str) -> SqliteResult<Vec<CategoryWithCount>> {
        self.with_db(|db| db.get_categories_for_search(search_query))
    }
    
    fn get_games_by_date_range(&self, days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_date_range(days_ago, limit, offset))
    }
    
    fn get_games_by_size_range(&self, min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_size_range(min_size, max_size, limit, offset))
    }
    
    fn get_games_by_categories_and_size(&self, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_categories_and_size(category_ids, min_size, max_size, limit, offset))
    }
    
    fn get_games_by_categories_and_time(&self, category_ids: &[i64], days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_categories_and_time(category_ids, days_ago, limit, offset))
    }
    
    fn get_games_by_size_and_time(&self, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_size_and_time(min_size, max_size, days_ago, limit, offset))
    }
    
    fn get_games_by_categories_size_and_time(&self, category_ids: &[i64], min_size: Option<i64>, max_size: Option<i64>, days_ago: i32, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_categories_size_and_time(category_ids, min_size, max_size, days_ago, limit, offset))
    }
    
    fn get_games_by_category(&self, category_id: i64, limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_category(category_id, limit, offset))
    }
    
    fn get_games_by_multiple_categories(&self, category_ids: &[i64], limit: i32, offset: i32) -> SqliteResult<Vec<Game>> {
        self.with_db(|db| db.get_games_by_multiple_categories(category_ids, limit, offset))
    }
    
    fn get_latest_game_date(&self) -> SqliteResult<Option<String>> {
        self.with_db(|db| db.get_latest_game_date())
    }
    
    fn get_settings(&self) -> SqliteResult<AppSettings> {
        self.with_db(|db| db.get_settings())
    }
    
    fn save_settings(&self, settings: &AppSettings) -> SqliteResult<()> {
        self.with_db(|db| db.save_settings(settings))
    }
    
    fn get_popular_repacks(&self, period: &str, limit: i32) -> SqliteResult<Vec<PopularRepack>> {
        self.with_db(|db| db.get_popular_repacks(period, limit))
    }
    
    fn get_popular_repacks_with_games(&self, period: &str, limit: i32) -> SqliteResult<Vec<PopularRepackWithGame>> {
        self.with_db(|db| db.get_popular_repacks_with_games(period, limit))
    }
    
    fn save_popular_repack(&self, url: &str, title: &str, image_url: Option<&str>, rank: i32, period: &str) -> SqliteResult<i64> {
        self.with_db(|db| db.save_popular_repack(url, title, image_url, rank, period))
    }
    
    fn clear_popular_repacks(&self, period: &str) -> SqliteResult<()> {
        self.with_db(|db| db.clear_popular_repacks(period))
    }
    
    fn update_popular_repack_links(&self, period: Option<&str>) -> SqliteResult<usize> {
        self.with_db(|db| db.update_popular_repack_links(period))
    }
    
    fn get_all_downloads(&self) -> SqliteResult<Vec<Download>> {
        self.with_db(|db| db.get_all_downloads())
    }
    
    fn get_download_by_info_hash(&self, info_hash: &str) -> SqliteResult<Option<Download>> {
        self.with_db(|db| db.get_download_by_info_hash(info_hash))
    }
    
    fn create_download(&self, repack_id: i64, game_title: &str, magnet_link: &str, info_hash: &str, save_path: &str) -> SqliteResult<i64> {
        self.with_db(|db| db.create_download(repack_id, game_title, magnet_link, info_hash, save_path))
    }
    
    fn update_download_status(&self, info_hash: &str, status: &str, error_message: Option<&str>) -> SqliteResult<()> {
        self.with_db(|db| db.update_download_status(info_hash, status, error_message))
    }
    
    fn delete_download(&self, info_hash: &str) -> SqliteResult<()> {
        self.with_db(|db| db.delete_download(info_hash))
    }
    
    fn check_table_exists(&self, table_name: &str) -> SqliteResult<bool> {
        self.with_db(|db| {
            let count: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table_name],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        })
    }
    
    fn check_url_exists(&self, url: &str) -> SqliteResult<bool> {
        self.with_db(|db| {
            let count: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM repacks WHERE url = ?1",
                [url],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        })
    }
    
    fn get_new_games_count(&self) -> SqliteResult<i64> {
        self.with_db(|db| {
            let count: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM repacks 
                 WHERE is_seen = 0
                 AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = repacks.id)",
                [],
                |row| row.get(0),
            )?;
            Ok(count)
        })
    }
    
    fn with_connection<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&Database) -> SqliteResult<R>
    {
        self.with_db(f)
    }
}

