// Example of SOLID-compliant command structure
// This demonstrates Dependency Inversion Principle

use crate::database::{CategoryWithCount, DatabaseStats, Game, GameDetails};
use super::database_service::DatabaseService;
use std::sync::Arc;
use tauri::State;

/// Application state with injected database service
/// This follows Dependency Inversion - depends on trait, not concrete type
pub struct AppStateV2 {
    pub db_service: Arc<dyn DatabaseService>,
}

/// Commands now depend on DatabaseService abstraction, not concrete Database
/// Benefits:
/// 1. Single database connection reused across all calls
/// 2. Easy to mock for testing
/// 3. Can swap implementations without changing command code
/// 4. Follows DRY principle - no repeated db initialization

#[tauri::command]
pub async fn search_games_v2(
    query: String,
    limit: i32,
    state: State<'_, AppStateV2>,
) -> Result<Vec<Game>, String> {
    // No more repeated db_path.lock() and Database::new() - just use the service!
    state.db_service
        .search_games(&query, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_games_v2(
    limit: i32,
    offset: i32,
    state: State<'_, AppStateV2>,
) -> Result<Vec<Game>, String> {
    state.db_service
        .get_all_games(limit, offset)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_game_details_v2(
    game_id: i64,
    state: State<'_, AppStateV2>,
) -> Result<GameDetails, String> {
    state.db_service
        .get_game_details(game_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_database_stats_v2(
    state: State<'_, AppStateV2>,
) -> Result<DatabaseStats, String> {
    state.db_service
        .get_stats()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_with_counts_v2(
    state: State<'_, AppStateV2>,
) -> Result<Vec<CategoryWithCount>, String> {
    state.db_service
        .get_categories_with_counts()
        .map_err(|e| e.to_string())
}

// ... other commands follow the same pattern

// Compare this to the old pattern:
// OLD (repeated ~50 times):
//   let db_path = state.db_path.lock().unwrap().clone();
//   let db = Database::new(db_path).map_err(|e| e.to_string())?;
//   db.search_games(&query, limit).map_err(|e| e.to_string())
//
// NEW (clean & reusable):
//   state.db_service.search_games(&query, limit).map_err(|e| e.to_string())

