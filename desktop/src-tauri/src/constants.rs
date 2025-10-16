// Shared application constants
// These constants are exposed to the frontend via Tauri commands

use serde::{Deserialize, Serialize};

// Database query limits
pub const LOAD_ALL_GAMES: i32 = 999999;
pub const DEFAULT_OFFSET: i32 = 0;

// Polling and timeouts (in milliseconds)
pub const POLLING_INTERVAL_MS: u64 = 2000;
pub const SEARCH_DEBOUNCE_MS: u64 = 300;

// Virtualization settings
pub const ITEM_HEIGHT: u32 = 30;
pub const OVERSCAN: u32 = 5;

// Popular repacks fetch limit (effectively "fetch all")
pub const POPULAR_FETCH_LIMIT: i32 = 9999;

// Auto-refresh intervals (in milliseconds)
pub const POPULAR_REFRESH_INTERVAL_MS: u64 = 3000;
pub const DISK_INFO_REFRESH_INTERVAL_MS: u64 = 30000;

// Database remote URL
pub const DATABASE_URL: &str = "http://157.230.16.45/repacks.db";

// Struct to export all constants at once to frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConstants {
    pub load_all_games: i32,
    pub default_offset: i32,
    pub polling_interval_ms: u64,
    pub search_debounce_ms: u64,
    pub item_height: u32,
    pub overscan: u32,
    pub popular_fetch_limit: i32,
    pub popular_refresh_interval_ms: u64,
    pub disk_info_refresh_interval_ms: u64,
    pub database_url: String,
}

impl AppConstants {
    pub fn new() -> Self {
        Self {
            load_all_games: LOAD_ALL_GAMES,
            default_offset: DEFAULT_OFFSET,
            polling_interval_ms: POLLING_INTERVAL_MS,
            search_debounce_ms: SEARCH_DEBOUNCE_MS,
            item_height: ITEM_HEIGHT,
            overscan: OVERSCAN,
            popular_fetch_limit: POPULAR_FETCH_LIMIT,
            popular_refresh_interval_ms: POPULAR_REFRESH_INTERVAL_MS,
            disk_info_refresh_interval_ms: DISK_INFO_REFRESH_INTERVAL_MS,
            database_url: DATABASE_URL.to_string(),
        }
    }
}

