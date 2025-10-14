use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub genres_tags: Option<String>,
    pub company: Option<String>,
    pub languages: Option<String>,
    pub original_size: Option<String>,
    pub repack_size: Option<String>,
    pub url: String,
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MagnetLink {
    pub id: i64,
    pub repack_id: i64,
    pub source: String,
    pub magnet: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameDetails {
    #[serde(flatten)]
    pub game: Game,
    pub magnet_links: Vec<MagnetLink>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn search_games(&self, query: &str, limit: i32) -> Result<Vec<Game>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, url, date 
             FROM repacks 
             WHERE title LIKE ?1 
             ORDER BY date DESC 
             LIMIT ?2"
        )?;

        let games = stmt
            .query_map(&[&search_pattern, &limit.to_string()], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    url: row.get(7)?,
                    date: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_all_games(&self, limit: i32, offset: i32) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, url, date 
             FROM repacks 
             ORDER BY date DESC 
             LIMIT ?1 OFFSET ?2"
        )?;

        let games = stmt
            .query_map(&[&limit.to_string(), &offset.to_string()], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    url: row.get(7)?,
                    date: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_game_details(&self, game_id: i64) -> Result<GameDetails> {
        // Get game info
        let game: Game = self.conn.query_row(
            "SELECT id, title, genres_tags, company, languages, original_size, repack_size, url, date 
             FROM repacks WHERE id = ?1",
            [game_id],
            |row| {
                Ok(Game {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genres_tags: row.get(2)?,
                    company: row.get(3)?,
                    languages: row.get(4)?,
                    original_size: row.get(5)?,
                    repack_size: row.get(6)?,
                    url: row.get(7)?,
                    date: row.get(8)?,
                })
            },
        )?;

        // Get magnet links
        let mut stmt = self.conn.prepare(
            "SELECT id, repack_id, source, magnet FROM magnet_links WHERE repack_id = ?1",
        )?;

        let magnet_links = stmt
            .query_map([game_id], |row| {
                Ok(MagnetLink {
                    id: row.get(0)?,
                    repack_id: row.get(1)?,
                    source: row.get(2)?,
                    magnet: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(GameDetails { game, magnet_links })
    }

    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let total_games: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM repacks", [], |row| row.get(0))?;

        let total_magnets: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM magnet_links", [], |row| row.get(0))?;

        Ok(DatabaseStats {
            total_games,
            total_magnets,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_games: i64,
    pub total_magnets: i64,
}
