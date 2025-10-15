use rusqlite::Result;
use super::models::{PopularRepack, PopularRepackWithGame, Game};

pub struct PopularQueries;

impl PopularQueries {
    pub fn save_popular_repack(conn: &rusqlite::Connection, url: &str, title: &str, image_url: Option<&str>, rank: i32, period: &str) -> Result<i64> {
        // First, try to find the repack_id by URL
        let repack_id: Option<i64> = conn.query_row(
            "SELECT id FROM repacks WHERE url = ?1",
            [url],
            |row| row.get(0),
        ).ok();
        
        // Insert or update popular repack
        conn.execute(
            "INSERT INTO popular_repacks (url, title, image_url, rank, period, repack_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
             ON CONFLICT(url, period) DO UPDATE SET
                title = excluded.title,
                image_url = excluded.image_url,
                rank = excluded.rank,
                repack_id = excluded.repack_id,
                updated_at = CURRENT_TIMESTAMP",
            (url, title, image_url, rank, period, repack_id),
        )?;
        
        // Return the ID
        let id: i64 = conn.query_row(
            "SELECT id FROM popular_repacks WHERE url = ?1 AND period = ?2",
            (url, period),
            |row| row.get(0),
        )?;
        
        Ok(id)
    }
    
    pub fn get_popular_repacks(conn: &rusqlite::Connection, period: &str, limit: i32) -> Result<Vec<PopularRepack>> {
        let mut stmt = conn.prepare(
            "SELECT id, url, title, image_url, rank, period, repack_id 
             FROM popular_repacks 
             WHERE period = ?1
             ORDER BY rank ASC
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
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(repacks)
    }
    
    pub fn get_popular_repacks_with_games(conn: &rusqlite::Connection, period: &str, limit: i32) -> Result<Vec<PopularRepackWithGame>> {
        let mut stmt = conn.prepare(
            "SELECT 
                pr.id, pr.url, pr.title, pr.image_url, pr.rank, pr.period,
                r.id, r.title, r.clean_name, r.genres_tags, r.company, r.languages, 
                r.original_size, r.repack_size, r.size, r.url, r.date, r.image_url
             FROM popular_repacks pr
             LEFT JOIN repacks r ON pr.repack_id = r.id
             WHERE pr.period = ?1
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
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(repacks)
    }
    
    pub fn clear_popular_repacks(conn: &rusqlite::Connection, period: &str) -> Result<()> {
        conn.execute("DELETE FROM popular_repacks WHERE period = ?1", [period])?;
        Ok(())
    }
    
    pub fn update_popular_repack_links(conn: &rusqlite::Connection, period: Option<&str>) -> Result<usize> {
        // Update repack_id for all popular repacks where URL matches
        let count = if let Some(p) = period {
            conn.execute(
                "UPDATE popular_repacks 
                 SET repack_id = (SELECT id FROM repacks WHERE repacks.url = popular_repacks.url)
                 WHERE period = ?1 AND EXISTS (SELECT 1 FROM repacks WHERE repacks.url = popular_repacks.url)",
                [p],
            )?
        } else {
            conn.execute(
                "UPDATE popular_repacks 
                 SET repack_id = (SELECT id FROM repacks WHERE repacks.url = popular_repacks.url)
                 WHERE EXISTS (SELECT 1 FROM repacks WHERE repacks.url = popular_repacks.url)",
                [],
            )?
        };
        
        Ok(count)
    }
}

