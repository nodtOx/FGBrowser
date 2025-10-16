use crate::database::{CategoryWithCount, Database, DatabaseStats, Game, GameDetails};
use super::utils::AppState;
use tauri::State;

#[tauri::command]
pub async fn search_games(
    query: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.search_games(&query, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_games(
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_all_games(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_game_details(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<GameDetails, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_game_details(game_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, AppState>,
) -> Result<DatabaseStats, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_with_counts(
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_with_counts().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_filtered_games(
    selected_category_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_filtered_games(&selected_category_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_time_filtered_games(
    days_ago: i32,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_time_filtered_games(days_ago).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_size_filtered_games(
    min_size: Option<i64>,
    max_size: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_size_filtered_games(min_size, max_size).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_size_and_time_filtered_games(
    min_size: Option<i64>,
    max_size: Option<i64>,
    days_ago: i32,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_size_and_time_filtered_games(min_size, max_size, days_ago).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories_for_search(
    search_query: String,
    state: State<'_, AppState>,
) -> Result<Vec<CategoryWithCount>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_categories_for_search(&search_query).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_date_range(
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_date_range(days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_size_range(
    min_size: Option<i64>,
    max_size: Option<i64>,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_size_range(min_size, max_size, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_categories_and_size(
    category_ids: Vec<i64>,
    min_size: Option<i64>,
    max_size: Option<i64>,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_categories_and_size(&category_ids, min_size, max_size, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_categories_and_time(
    category_ids: Vec<i64>,
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_categories_and_time(&category_ids, days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_size_and_time(
    min_size: Option<i64>,
    max_size: Option<i64>,
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_size_and_time(min_size, max_size, days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_categories_size_and_time(
    category_ids: Vec<i64>,
    min_size: Option<i64>,
    max_size: Option<i64>,
    days_ago: i32,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_categories_size_and_time(&category_ids, min_size, max_size, days_ago, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_category(
    category_id: i64,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_category(category_id, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_games_by_multiple_categories(
    category_ids: Vec<i64>,
    limit: i32,
    offset: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    db.get_games_by_multiple_categories(&category_ids, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_category_cache() -> Result<(), String> {
    Database::clear_category_cache();
    println!("🧹 Category cache cleared");
    Ok(())
}

#[tauri::command]
pub async fn is_database_empty(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    
    // Check if file exists
    if !db_path.exists() {
        return Ok(true);
    }
    
    // Check if database has repacks table and any games
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    
    // Check if repacks table exists
    let table_exists: Result<i64, _> = db.conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='repacks'",
        [],
        |row| row.get(0),
    );
    
    match table_exists {
        Ok(count) if count > 0 => {
            // Table exists, check if it has data
            let stats = db.get_stats().map_err(|e| e.to_string())?;
            Ok(stats.total_games == 0)
        }
        _ => {
            // Table doesn't exist, database is empty
            Ok(true)
        }
    }
}

