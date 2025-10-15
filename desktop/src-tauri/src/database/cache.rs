use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use super::models::Game;

// Global query cache for category filtering
pub static CATEGORY_QUERY_CACHE: LazyLock<Mutex<HashMap<String, Vec<Game>>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub const MAX_CACHE_ENTRIES: usize = 100;

// Cache management functions
pub fn get_cache_key(category_ids: &[i64]) -> String {
    let mut sorted_ids = category_ids.to_vec();
    sorted_ids.sort();
    sorted_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")
}

pub fn get_from_cache(cache_key: &str) -> Option<Vec<Game>> {
    let cache = CATEGORY_QUERY_CACHE.lock().ok()?;
    cache.get(cache_key).cloned()
}

pub fn add_to_cache(cache_key: String, games: Vec<Game>) {
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

