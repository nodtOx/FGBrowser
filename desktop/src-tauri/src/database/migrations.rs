use rusqlite::{Connection, Result};

pub fn migrate_repacks_image_url(conn: &Connection) -> Result<()> {
    // Check if image_url column exists
    let column_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('repacks') WHERE name = 'image_url'",
        [],
        |row| row.get(0),
    );
    
    match column_exists {
        Ok(count) if count == 0 => {
            println!("🔄 Adding image_url column to repacks table...");
            
            // Add image_url column
            conn.execute(
                "ALTER TABLE repacks ADD COLUMN image_url TEXT",
                [],
            )?;
            
            println!("✅ Migration completed! Added image_url column to repacks");
        }
        Ok(_) => {
            // Column already exists, do nothing
        }
        Err(e) => {
            eprintln!("Warning: Could not check for image_url column: {}", e);
        }
    }
    
    Ok(())
}

pub fn migrate_repacks_clean_name(conn: &Connection) -> Result<()> {
    // Check if clean_name column exists
    let column_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('repacks') WHERE name = 'clean_name'",
        [],
        |row| row.get(0),
    );
    
    match column_exists {
        Ok(count) if count == 0 => {
            println!("🔄 Adding clean_name column to repacks table...");
            
            // Add clean_name column
            conn.execute(
                "ALTER TABLE repacks ADD COLUMN clean_name TEXT",
                [],
            )?;
            
            println!("✅ Migration completed! Added clean_name column to repacks");
        }
        Ok(_) => {
            // Column already exists, do nothing
        }
        Err(e) => {
            eprintln!("Warning: Could not check for clean_name column: {}", e);
        }
    }
    
    Ok(())
}

pub fn populate_clean_names(conn: &Connection) -> Result<()> {
    // Check if there are any games without clean names
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM repacks WHERE clean_name IS NULL OR clean_name = ''",
        [],
        |row| row.get(0),
    )?;
    
    if count > 0 {
        println!("🔄 Populating clean names for {} existing games...", count);
        
        // Get all games that need clean names
        let mut stmt = conn.prepare("SELECT id, title FROM repacks WHERE clean_name IS NULL OR clean_name = ''")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        
        let mut updated_count = 0;
        for row in rows {
            if let Ok((id, title)) = row {
                // Generate clean name using the crawler function
                let clean_name = crate::crawler::clean_game_title(&title);
                
                // Update the record
                conn.execute(
                    "UPDATE repacks SET clean_name = ?1 WHERE id = ?2",
                    (&clean_name, id),
                )?;
                
                updated_count += 1;
                
                // Print progress every 50 games
                if updated_count % 50 == 0 {
                    println!("  ✓ Processed {} games...", updated_count);
                }
            }
        }
        
        println!("✅ Clean names populated for {} games", updated_count);
    }
    
    Ok(())
}

pub fn migrate_popular_repacks_period(conn: &Connection) -> Result<()> {
    // Check if period column exists
    let column_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('popular_repacks') WHERE name = 'period'",
        [],
        |row| row.get(0),
    );
    
    match column_exists {
        Ok(count) if count == 0 => {
            println!("🔄 Migrating popular_repacks table to add period column...");
            
            // Add period column with default value 'month'
            conn.execute(
                "ALTER TABLE popular_repacks ADD COLUMN period TEXT NOT NULL DEFAULT 'month'",
                [],
            )?;
            
            // Drop old unique constraint and create new composite one
            // SQLite doesn't support DROP CONSTRAINT, so we need to recreate the table
            conn.execute(
                "CREATE TABLE popular_repacks_new (
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
            
            // Copy data from old table
            conn.execute(
                "INSERT INTO popular_repacks_new (id, url, title, image_url, rank, period, repack_id, created_at, updated_at)
                 SELECT id, url, title, image_url, rank, period, repack_id, created_at, updated_at FROM popular_repacks",
                [],
            )?;
            
            // Drop old table
            conn.execute("DROP TABLE popular_repacks", [])?;
            
            // Rename new table
            conn.execute("ALTER TABLE popular_repacks_new RENAME TO popular_repacks", [])?;
            
            // Recreate indexes
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_popular_repacks_period_rank ON popular_repacks(period, rank)",
                [],
            )?;
            
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_popular_repacks_url_period ON popular_repacks(url, period)",
                [],
            )?;
            
            println!("✅ Migration completed! Added period column to popular_repacks");
        }
        Ok(_) => {
            // Column already exists, do nothing
        }
        Err(e) => {
            eprintln!("Warning: Could not check for period column: {}", e);
        }
    }
    
    Ok(())
}

pub fn migrate_categories_data(conn: &Connection) -> Result<()> {
    // Check if migration is needed (if categories table is empty but repacks have genres_tags)
    let categories_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM categories",
        [],
        |row| row.get(0),
    )?;
    
    let repacks_with_genres: i64 = conn.query_row(
        "SELECT COUNT(*) FROM repacks WHERE genres_tags IS NOT NULL AND genres_tags != ''",
        [],
        |row| row.get(0),
    )?;
    
    // If categories table is empty but we have repacks with genres, run migration
    if categories_count == 0 && repacks_with_genres > 0 {
        println!("🔄 Migrating genre/tag data to new category structure...");
        
        // Get all unique categories from existing data
        let mut stmt = conn.prepare(
            "SELECT id, genres_tags FROM repacks WHERE genres_tags IS NOT NULL AND genres_tags != ''"
        )?;
        
        let mut category_map = std::collections::HashMap::<String, i64>::new();
        
        let repack_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        
        for repack_row in repack_rows {
            let (repack_id, genres_tags) = repack_row?;
            
            // Split by comma and clean up
            let categories: Vec<String> = genres_tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            for category_name in categories {
                // Insert category if not exists and get ID
                let category_id = if let Some(&existing_id) = category_map.get(&category_name) {
                    existing_id
                } else {
                    // Insert new category
                    conn.execute(
                        "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
                        [&category_name],
                    )?;
                    
                    // Get the category ID
                    let category_id: i64 = conn.query_row(
                        "SELECT id FROM categories WHERE name = ?1",
                        [&category_name],
                        |row| row.get(0),
                    )?;
                    
                    category_map.insert(category_name.clone(), category_id);
                    category_id
                };
                
                // Insert game-category relationship
                conn.execute(
                    "INSERT OR IGNORE INTO game_categories (repack_id, category_id) VALUES (?1, ?2)",
                    [repack_id, category_id],
                )?;
            }
        }
        
        println!("✅ Migration completed! Created {} categories", category_map.len());
    }
    
    Ok(())
}

