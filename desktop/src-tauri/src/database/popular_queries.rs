use rusqlite::Result;
use super::models::{PopularRepack, PopularRepackWithGame, Game};

pub struct PopularQueries;

impl PopularQueries {
    pub fn save_popular_repack(conn: &rusqlite::Connection, url: &str, _title: &str, _image_url: Option<&str>, rank: i32, period: &str) -> Result<i64> {
        // Find the repack_id by URL - this is now REQUIRED
        let repack_id: Option<i64> = conn.query_row(
            "SELECT id FROM repacks WHERE url = ?1",
            [url],
            |row| row.get(0),
        ).ok();
        
        // If we don't have a repack_id, we can't save it (normalized schema requires it)
        let repack_id = match repack_id {
            Some(id) => id,
            None => {
                // Game not yet in repacks table, skip for now
                // It will be added when the game is crawled
                return Ok(0); // Return 0 to indicate not saved
            }
        };
        
        // Insert or update popular repack (only storing repack_id, rank, period)
        conn.execute(
            "INSERT INTO popular_repacks (repack_id, rank, period, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(repack_id, period) DO UPDATE SET
                rank = excluded.rank,
                updated_at = CURRENT_TIMESTAMP",
            (repack_id, rank, period),
        )?;
        
        // Return the ID
        let id: i64 = conn.query_row(
            "SELECT id FROM popular_repacks WHERE repack_id = ?1 AND period = ?2",
            (repack_id, period),
            |row| row.get(0),
        )?;
        
        Ok(id)
    }
    
    pub fn get_popular_repacks(conn: &rusqlite::Connection, period: &str, limit: i32) -> Result<Vec<PopularRepack>> {
        let mut stmt = conn.prepare(
            "SELECT pr.id, r.url, r.title, r.image_url, pr.rank, pr.period, pr.repack_id, pr.created_at 
             FROM popular_repacks pr
             INNER JOIN repacks r ON pr.repack_id = r.id
             WHERE pr.period = ?1
             ORDER BY pr.rank ASC
             LIMIT ?2"
        )?;

        let repacks = stmt
            .query_map([period, &limit.to_string()], |row| {
                Ok(PopularRepack {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    image_url: row.get(3)?,
                    rank: row.get(4)?,
                    period: row.get(5)?,
                    repack_id: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(repacks)
    }
    
    pub fn get_popular_repacks_with_games(conn: &rusqlite::Connection, period: &str, limit: i32) -> Result<Vec<PopularRepackWithGame>> {
        let mut stmt = conn.prepare(
            "SELECT 
                pr.id, r.url, r.title, r.image_url, pr.rank, pr.period,
                r.id, r.title, r.clean_name, r.genres_tags, r.company, r.languages, 
                r.original_size, r.repack_size, r.size, r.url, r.date, r.image_url,
                pr.created_at,
                CASE WHEN julianday('now') - julianday(pr.created_at) <= 7 THEN 1 ELSE 0 END as is_new
             FROM popular_repacks pr
             INNER JOIN repacks r ON pr.repack_id = r.id
             WHERE pr.period = ?1
             AND EXISTS (SELECT 1 FROM magnet_links WHERE magnet_links.repack_id = r.id)
             ORDER BY pr.rank ASC
             LIMIT ?2"
        )?;

        let repacks = stmt
            .query_map([period, &limit.to_string()], |row| {
                let game = if let Ok(game_id) = row.get::<_, i64>(6) {
                    Some(Game {
                        id: game_id,
                        title: row.get(7)?,
                        clean_name: row.get(8)?,
                        genres_tags: row.get(9)?,
                        company: row.get(10)?,
                        languages: row.get(11)?,
                        original_size: row.get(12)?,
                        repack_size: row.get(13)?,
                        size: row.get(14)?,
                        url: row.get(15)?,
                        date: row.get(16)?,
                        image_url: row.get(17)?,
                    })
                } else {
                    None
                };
                
                Ok(PopularRepackWithGame {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    image_url: row.get(3)?,
                    rank: row.get(4)?,
                    period: row.get(5)?,
                    game,
                    created_at: row.get(18)?,
                    is_new: row.get::<_, i32>(19)? == 1,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(repacks)
    }
    
    pub fn clear_popular_repacks(conn: &rusqlite::Connection, period: &str) -> Result<()> {
        conn.execute("DELETE FROM popular_repacks WHERE period = ?1", [period])?;
        Ok(())
    }
    
    pub fn update_popular_repack_links(conn: &rusqlite::Connection, _period: Option<&str>) -> Result<usize> {
        // This function is no longer needed with the normalized schema
        // All popular repacks are now inserted with a valid repack_id
        // Keeping for backward compatibility but returning 0
        Ok(0)
    }

    // Get count of unseen popular games for a period
    pub fn get_unseen_count(conn: &rusqlite::Connection, period: &str, last_viewed: Option<&str>) -> Result<i64> {
        let count = match last_viewed {
            Some(timestamp) => {
                conn.query_row(
                    "SELECT COUNT(*) FROM popular_repacks 
                     WHERE period = ?1 AND created_at > ?2",
                    [period, timestamp],
                    |row| row.get(0),
                )?
            }
            None => {
                // If never viewed, count all games
                conn.query_row(
                    "SELECT COUNT(*) FROM popular_repacks WHERE period = ?1",
                    [period],
                    |row| row.get(0),
                )?
            }
        };
        Ok(count)
    }

    // Get total unseen count across all periods
    pub fn get_total_unseen_count(
        conn: &rusqlite::Connection,
        month_last_viewed: Option<&str>,
        year_last_viewed: Option<&str>,
        award_last_viewed: Option<&str>,
    ) -> Result<i64> {
        let month_count = Self::get_unseen_count(conn, "month", month_last_viewed)?;
        let year_count = Self::get_unseen_count(conn, "year", year_last_viewed)?;
        let award_count = Self::get_unseen_count(conn, "award", award_last_viewed)?;
        Ok(month_count + year_count + award_count)
    }
}

