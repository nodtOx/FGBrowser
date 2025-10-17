use rusqlite::{Connection, Result};

pub fn migrate_repacks_image_url(conn: &Connection) -> Result<()> {
    // Check if image_url column exists
    let column_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('repacks') WHERE name = 'image_url'",
        [],
        |row| row.get(0),
    );
    
    match column_exists {
        Ok(0) => {
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
        Ok(0) => {
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
        
        // Collect all rows first to avoid borrowing issues with the transaction
        let games: Vec<(i64, String)> = rows.collect::<Result<Vec<_>>>()?;
        
        // Use a transaction for bulk updates (100-1000x faster!)
        let tx = conn.unchecked_transaction()?;
        {
            let mut update_stmt = tx.prepare("UPDATE repacks SET clean_name = ?1 WHERE id = ?2")?;
            
            let mut updated_count = 0;
            for (id, title) in games {
                // Generate clean name using the crawler function
                let clean_name = crate::crawler::clean_game_title(&title);
                
                // Update the record
                update_stmt.execute((&clean_name, id))?;
                
                updated_count += 1;
                
                // Print progress every 50 games
                if updated_count % 50 == 0 {
                    println!("  ✓ Processed {} games...", updated_count);
                }
            }
            
            println!("✅ Clean names populated for {} games", updated_count);
        }
        tx.commit()?;
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
        Ok(0) => {
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

pub fn migrate_normalize_popular_repacks(conn: &Connection) -> Result<()> {
    // Check if the table still has the redundant columns
    let has_url: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('popular_repacks') WHERE name = 'url'",
        [],
        |row| row.get(0),
    );
    
    match has_url {
        Ok(count) if count > 0 => {
            println!("🔄 Normalizing popular_repacks table to remove duplicate data...");
            
            // Create new normalized table
            conn.execute(
                "CREATE TABLE popular_repacks_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    repack_id INTEGER NOT NULL,
                    rank INTEGER NOT NULL,
                    period TEXT NOT NULL DEFAULT 'month',
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (repack_id) REFERENCES repacks (id) ON DELETE CASCADE,
                    UNIQUE(repack_id, period)
                )",
                [],
            )?;
            
            // Copy data from old table (only keeping repack_id, rank, period)
            // Only migrate records that have a valid repack_id
            conn.execute(
                "INSERT INTO popular_repacks_new (id, repack_id, rank, period, created_at, updated_at)
                 SELECT id, repack_id, rank, period, created_at, updated_at 
                 FROM popular_repacks 
                 WHERE repack_id IS NOT NULL",
                [],
            )?;
            
            // Drop old table
            conn.execute("DROP TABLE popular_repacks", [])?;
            
            // Rename new table
            conn.execute("ALTER TABLE popular_repacks_new RENAME TO popular_repacks", [])?;
            
            // Create indexes
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_popular_repacks_period_rank ON popular_repacks(period, rank)",
                [],
            )?;
            
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_popular_repacks_repack_id ON popular_repacks(repack_id)",
                [],
            )?;
            
            println!("✅ Migration completed! Normalized popular_repacks table");
            println!("   - Removed duplicate columns: url, title, image_url");
            println!("   - Data now comes from repacks table via JOIN");
        }
        Ok(_) => {
            // Already normalized, do nothing
        }
        Err(e) => {
            eprintln!("Warning: Could not check for url column: {}", e);
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
        
        let repack_rows: Vec<(i64, String)> = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?.collect::<Result<Vec<_>>>()?;
        
        // Use transaction for bulk operations
        let tx = conn.unchecked_transaction()?;
        {
            let mut category_map = std::collections::HashMap::<String, i64>::new();
            
            let mut insert_category_stmt = tx.prepare("INSERT OR IGNORE INTO categories (name) VALUES (?1)")?;
            let mut get_category_stmt = tx.prepare("SELECT id FROM categories WHERE name = ?1")?;
            let mut insert_game_category_stmt = tx.prepare("INSERT OR IGNORE INTO game_categories (repack_id, category_id) VALUES (?1, ?2)")?;
            
            for (repack_id, genres_tags) in repack_rows {
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
                        insert_category_stmt.execute([&category_name])?;
                        
                        // Get the category ID
                        let category_id: i64 = get_category_stmt.query_row([&category_name], |row| row.get(0))?;
                        
                        category_map.insert(category_name.clone(), category_id);
                        category_id
                    };
                    
                    // Insert game-category relationship
                    insert_game_category_stmt.execute([repack_id, category_id])?;
                }
            }
            
            println!("✅ Migration completed! Created {} categories", category_map.len());
        }
        tx.commit()?;
    }
    
    Ok(())
}

pub fn migrate_cleanup_malformed_categories(conn: &Connection) -> Result<()> {
    
    // Get all categories that contain multi-line content (likely malformed)
    let mut stmt = conn.prepare(
        "SELECT id, name FROM categories WHERE name LIKE '%\n%' OR name LIKE '%Company%' OR name LIKE '%Languages:%' OR name LIKE '%Size:%'"
    )?;
    
    let malformed_categories = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    
    let mut cleaned_count = 0;
    let mut removed_count = 0;
    
    for category_row in malformed_categories {
        let (category_id, malformed_name) = category_row?;
        
        // Extract the legitimate genre (first line)
        let legitimate_genre = if let Some(first_line) = malformed_name.lines().next() {
            first_line.trim().to_string()
        } else {
            continue;
        };
        
        // Skip if the legitimate genre is empty or looks like metadata
        if legitimate_genre.is_empty() || 
           legitimate_genre.contains("Company") || 
           legitimate_genre.contains("Languages:") || 
           legitimate_genre.contains("Size:") ||
           legitimate_genre.contains("iMDB") ||
           legitimate_genre.contains("Director") ||
           legitimate_genre.contains("Video format") {
            // This is a pure metadata category, remove it entirely
            conn.execute("DELETE FROM game_categories WHERE category_id = ?", [&category_id.to_string()])?;
            conn.execute("DELETE FROM categories WHERE id = ?", [&category_id.to_string()])?;
            removed_count += 1;
            continue;
        }
        
        // Check if a clean category with this name already exists
        let existing_id: Result<i64, _> = conn.query_row(
            "SELECT id FROM categories WHERE name = ?",
            [&legitimate_genre],
            |row| row.get(0),
        );
        
        match existing_id {
            Ok(existing_category_id) => {
                // Clean category already exists, migrate all game associations
                conn.execute(
                    "UPDATE game_categories SET category_id = ? WHERE category_id = ?",
                    [&existing_category_id.to_string(), &category_id.to_string()],
                )?;
                
                // Remove the malformed category
                conn.execute("DELETE FROM categories WHERE id = ?", [&category_id.to_string()])?;
                cleaned_count += 1;
            }
            Err(_) => {
                // No clean category exists, just rename this one
                conn.execute(
                    "UPDATE categories SET name = ? WHERE id = ?",
                    [&legitimate_genre, &category_id.to_string()],
                )?;
                cleaned_count += 1;
            }
        }
    }
    
    if cleaned_count > 0 || removed_count > 0 {
        println!("✅ Cleaned {} malformed categories, removed {} metadata categories", cleaned_count, removed_count);
    }
    
    Ok(())
}

pub fn migrate_normalize_genre_variations(conn: &Connection) -> Result<()> {
    
    // Normalize "Shoot 'Em Up" variations
    let shoot_em_up_variations = vec![
        "Shoot 'em Up",
        "Shoot 'em up", 
        "Shoot 'em upm Isometric",
        "Shoot'Em-Up",
        "Shoot-'Em-Up",
        "Shoot'Em-Up",
    ];
    
    // Check if standard "Shoot 'Em Up" category exists
    let standard_name = "Shoot 'Em Up";
    let standard_exists: Result<i64, _> = conn.query_row(
        "SELECT id FROM categories WHERE name = ?",
        [standard_name],
        |row| row.get(0),
    );
    
    let standard_id = match standard_exists {
        Ok(id) => id,
        Err(_) => {
            // Create the standard category
            conn.execute("INSERT INTO categories (name) VALUES (?)", [standard_name])?;
            conn.query_row("SELECT id FROM categories WHERE name = ?", [standard_name], |row| row.get(0))?
        }
    };
    
    let mut normalized_count = 0;
    
    for variation in shoot_em_up_variations {
        // Check if this variation exists
        let variation_id: Result<i64, _> = conn.query_row(
            "SELECT id FROM categories WHERE name = ?",
            [variation],
            |row| row.get(0),
        );
        
        if let Ok(vid) = variation_id {
            // Move all game associations to the standard category
            conn.execute(
                "UPDATE game_categories SET category_id = ? WHERE category_id = ?",
                [&standard_id.to_string(), &vid.to_string()],
            )?;
            
            // Remove the variation
            conn.execute("DELETE FROM categories WHERE id = ?", [&vid.to_string()])?;
            normalized_count += 1;
        }
    }
    
    if normalized_count > 0 {
        println!("✅ Normalized {} genre variations", normalized_count);
    }
    Ok(())
}

