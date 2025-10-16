use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub clean_name: Option<String>,
    pub genres_tags: Option<String>,
    pub company: Option<String>,
    pub languages: Option<String>,
    pub original_size: Option<String>,
    pub repack_size: Option<String>,
    pub size: Option<i64>, // Size in MB (parsed from repack_size)
    pub url: String,
    pub date: Option<String>,
    pub image_url: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PopularRepack {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub image_url: Option<String>,
    pub rank: i32,
    pub period: String, // 'month' or 'year'
    pub repack_id: Option<i64>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PopularRepackWithGame {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub image_url: Option<String>,
    pub rank: i32,
    pub period: String, // 'month' or 'year'
    pub game: Option<Game>,
    pub created_at: Option<String>,
    pub is_new: bool, // true if added in last 7 days
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Download {
    pub id: i64,
    pub repack_id: i64,
    pub game_title: String,
    pub magnet_link: String,
    pub info_hash: String,
    pub status: String, // 'queued', 'downloading', 'seeding', 'paused', 'completed', 'error'
    pub save_path: String,
    pub total_size: i64, // bytes
    pub downloaded_bytes: i64,
    pub uploaded_bytes: i64,
    pub download_speed: i64, // bytes/sec
    pub upload_speed: i64, // bytes/sec
    pub progress: f64, // 0.0 to 100.0
    pub peers: i32,
    pub seeds: i32,
    pub eta_seconds: Option<i64>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

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

    // Popular last viewed timestamps (ISO 8601 strings)
    pub popular_month_last_viewed: Option<String>,
    pub popular_year_last_viewed: Option<String>,
    pub popular_award_last_viewed: Option<String>,
}

