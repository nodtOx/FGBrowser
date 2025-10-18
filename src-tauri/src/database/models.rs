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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    // General
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub close_to_tray: bool,
    #[serde(default = "default_true")]
    pub notifications: bool,

    // Download
    #[serde(default)]
    pub download_path: String,
    #[serde(default = "default_max_downloads")]
    pub max_simultaneous_downloads: i32,
    #[serde(default = "default_true")]
    pub auto_start_downloads: bool,
    #[serde(default = "default_true")]
    pub seed_after_complete: bool,
    #[serde(default = "default_seed_ratio")]
    pub seed_ratio: f64,

    // Network
    #[serde(default)]
    pub max_download_speed: i32,
    #[serde(default = "default_upload_speed")]
    pub max_upload_speed: i32,
    #[serde(default = "default_port")]
    pub port: i32,
    #[serde(default = "default_true")]
    pub use_upnp: bool,
    #[serde(default = "default_true")]
    pub use_dht: bool,

    // Appearance
    #[serde(default = "default_font_size")]
    pub font_size: i32,
    #[serde(default)]
    pub compact_mode: bool,
    #[serde(default = "default_true")]
    pub show_thumbnails: bool,
    #[serde(default = "default_true")]
    pub animations_enabled: bool,

    // Database
    #[serde(default)]
    pub db_path: String,
    #[serde(default)]
    pub auto_refresh: bool,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: i32,

    // Popular last viewed timestamps (ISO 8601 strings)
    pub popular_week_last_viewed: Option<String>,
    pub popular_today_last_viewed: Option<String>,
    pub popular_month_last_viewed: Option<String>,
    pub popular_year_last_viewed: Option<String>,
    pub popular_award_last_viewed: Option<String>,
}

// Default value helpers for serde
fn default_true() -> bool { true }
fn default_max_downloads() -> i32 { 3 }
fn default_seed_ratio() -> f64 { 1.5 }
fn default_upload_speed() -> i32 { 500 }
fn default_port() -> i32 { 6881 }
fn default_font_size() -> i32 { 14 }
fn default_refresh_interval() -> i32 { 24 }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            minimize_to_tray: false,
            close_to_tray: false,
            notifications: true,
            download_path: String::new(),
            max_simultaneous_downloads: 3,
            auto_start_downloads: true,
            seed_after_complete: true,
            seed_ratio: 1.5,
            max_download_speed: 0,
            max_upload_speed: 500,
            port: 6881,
            use_upnp: true,
            use_dht: true,
            font_size: 14,
            compact_mode: false,
            show_thumbnails: true,
            animations_enabled: true,
            db_path: String::new(),
            auto_refresh: false,
            refresh_interval: 24,
            popular_week_last_viewed: None,
            popular_today_last_viewed: None,
            popular_month_last_viewed: None,
            popular_year_last_viewed: None,
            popular_award_last_viewed: None,
        }
    }
}

