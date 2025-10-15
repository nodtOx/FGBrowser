use rusqlite::Result;
use super::models::CategoryWithCount;

pub struct CategoryQueries;

impl CategoryQueries {
    pub fn get_categories_with_counts(conn: &rusqlite::Connection) -> Result<Vec<CategoryWithCount>> {
        let mut stmt = conn.prepare(
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
    pub fn get_categories_for_filtered_games(conn: &rusqlite::Connection, selected_category_ids: &[i64]) -> Result<Vec<CategoryWithCount>> {
        if selected_category_ids.is_empty() {
            // If no categories selected, return all categories
            return Self::get_categories_with_counts(conn);
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

        let mut stmt = conn.prepare(&query)?;
        
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
    
    // Get categories that appear in games filtered by time
    pub fn get_categories_for_time_filtered_games(conn: &rusqlite::Connection, days_ago: i32) -> Result<Vec<CategoryWithCount>> {
        let query = 
            "SELECT c.id, c.name, COUNT(DISTINCT r.id) as game_count
             FROM categories c
             JOIN game_categories gc ON c.id = gc.category_id
             JOIN repacks r ON gc.repack_id = r.id
             WHERE r.date >= date('now', '-' || ? || ' days')
             GROUP BY c.id, c.name
             HAVING game_count > 0
             ORDER BY game_count DESC, c.name ASC";

        let mut stmt = conn.prepare(query)?;
        
        let categories = stmt
            .query_map([days_ago], |row| {
                Ok(CategoryWithCount {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    game_count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(categories)
    }
    
    // Get categories that appear in games filtered by size
    pub fn get_categories_for_size_filtered_games(conn: &rusqlite::Connection, min_size: Option<i64>, max_size: Option<i64>) -> Result<Vec<CategoryWithCount>> {
        let mut query = 
            "SELECT c.id, c.name, COUNT(DISTINCT r.id) as game_count
             FROM categories c
             JOIN game_categories gc ON c.id = gc.category_id
             JOIN repacks r ON gc.repack_id = r.id
             WHERE 1=1".to_string();
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(min) = min_size {
            query.push_str(" AND r.size >= ?");
            params.push(Box::new(min));
        }
        
        if let Some(max) = max_size {
            query.push_str(" AND r.size <= ?");
            params.push(Box::new(max));
        }
        
        query.push_str(" GROUP BY c.id, c.name HAVING game_count > 0 ORDER BY game_count DESC, c.name ASC");

        let mut stmt = conn.prepare(&query)?;
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
    
    // Get categories that appear in games filtered by size AND time
    pub fn get_categories_for_size_and_time_filtered_games(conn: &rusqlite::Connection, min_size: Option<i64>, max_size: Option<i64>, days_ago: i32) -> Result<Vec<CategoryWithCount>> {
        let mut query = 
            "SELECT c.id, c.name, COUNT(DISTINCT r.id) as game_count
             FROM categories c
             JOIN game_categories gc ON c.id = gc.category_id
             JOIN repacks r ON gc.repack_id = r.id
             WHERE r.date >= date('now', '-' || ? || ' days')".to_string();
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params.push(Box::new(days_ago));
        
        if let Some(min) = min_size {
            query.push_str(" AND r.size >= ?");
            params.push(Box::new(min));
        }
        
        if let Some(max) = max_size {
            query.push_str(" AND r.size <= ?");
            params.push(Box::new(max));
        }
        
        query.push_str(" GROUP BY c.id, c.name HAVING game_count > 0 ORDER BY game_count DESC, c.name ASC");

        let mut stmt = conn.prepare(&query)?;
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
}

