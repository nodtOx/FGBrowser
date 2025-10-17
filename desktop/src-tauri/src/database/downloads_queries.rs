use rusqlite::Result;
use super::models::Download;

pub struct DownloadQueries;

pub struct DownloadProgress<'a> {
    pub info_hash: &'a str,
    pub total_size: i64,
    pub downloaded_bytes: i64,
    pub uploaded_bytes: i64,
    pub download_speed: i64,
    pub upload_speed: i64,
    pub progress: f64,
    pub peers: i32,
    pub seeds: i32,
    pub eta_seconds: Option<i64>,
}

impl DownloadQueries {
    pub fn get_all_downloads(conn: &rusqlite::Connection) -> Result<Vec<Download>> {
        let mut stmt = conn.prepare(
            "SELECT id, repack_id, game_title, magnet_link, info_hash, status, save_path,
                    total_size, downloaded_bytes, uploaded_bytes, download_speed, upload_speed,
                    progress, peers, seeds, eta_seconds, error_message, started_at, completed_at
             FROM downloads
             ORDER BY created_at DESC"
        )?;

        let downloads = stmt
            .query_map([], |row| {
                Ok(Download {
                    id: row.get(0)?,
                    repack_id: row.get(1)?,
                    game_title: row.get(2)?,
                    magnet_link: row.get(3)?,
                    info_hash: row.get(4)?,
                    status: row.get(5)?,
                    save_path: row.get(6)?,
                    total_size: row.get(7)?,
                    downloaded_bytes: row.get(8)?,
                    uploaded_bytes: row.get(9)?,
                    download_speed: row.get(10)?,
                    upload_speed: row.get(11)?,
                    progress: row.get(12)?,
                    peers: row.get(13)?,
                    seeds: row.get(14)?,
                    eta_seconds: row.get(15)?,
                    error_message: row.get(16)?,
                    started_at: row.get(17)?,
                    completed_at: row.get(18)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(downloads)
    }

    pub fn get_download_by_info_hash(conn: &rusqlite::Connection, info_hash: &str) -> Result<Option<Download>> {
        let result = conn.query_row(
            "SELECT id, repack_id, game_title, magnet_link, info_hash, status, save_path,
                    total_size, downloaded_bytes, uploaded_bytes, download_speed, upload_speed,
                    progress, peers, seeds, eta_seconds, error_message, started_at, completed_at
             FROM downloads
             WHERE info_hash = ?1",
            [info_hash],
            |row| {
                Ok(Download {
                    id: row.get(0)?,
                    repack_id: row.get(1)?,
                    game_title: row.get(2)?,
                    magnet_link: row.get(3)?,
                    info_hash: row.get(4)?,
                    status: row.get(5)?,
                    save_path: row.get(6)?,
                    total_size: row.get(7)?,
                    downloaded_bytes: row.get(8)?,
                    uploaded_bytes: row.get(9)?,
                    download_speed: row.get(10)?,
                    upload_speed: row.get(11)?,
                    progress: row.get(12)?,
                    peers: row.get(13)?,
                    seeds: row.get(14)?,
                    eta_seconds: row.get(15)?,
                    error_message: row.get(16)?,
                    started_at: row.get(17)?,
                    completed_at: row.get(18)?,
                })
            },
        );

        match result {
            Ok(download) => Ok(Some(download)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn create_download(
        conn: &rusqlite::Connection,
        repack_id: i64,
        game_title: &str,
        magnet_link: &str,
        info_hash: &str,
        save_path: &str,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO downloads (repack_id, game_title, magnet_link, info_hash, save_path, status, started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'queued', CURRENT_TIMESTAMP)",
            [
                &repack_id.to_string(),
                game_title,
                magnet_link,
                info_hash,
                save_path,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn update_download_status(
        conn: &rusqlite::Connection,
        info_hash: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<()> {
        if status == "completed" {
            conn.execute(
                "UPDATE downloads 
                 SET status = ?1, error_message = ?2, completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                 WHERE info_hash = ?3",
                [&status, &error_message.unwrap_or(""), &info_hash],
            )?;
        } else {
            conn.execute(
                "UPDATE downloads 
                 SET status = ?1, error_message = ?2, updated_at = CURRENT_TIMESTAMP
                 WHERE info_hash = ?3",
                [&status, &error_message.unwrap_or(""), &info_hash],
            )?;
        }
        Ok(())
    }

    pub fn update_download_progress(
        conn: &rusqlite::Connection,
        progress: &DownloadProgress,
    ) -> Result<()> {
        conn.execute(
            "UPDATE downloads 
             SET total_size = ?1, downloaded_bytes = ?2, uploaded_bytes = ?3,
                 download_speed = ?4, upload_speed = ?5, progress = ?6,
                 peers = ?7, seeds = ?8, eta_seconds = ?9, updated_at = CURRENT_TIMESTAMP
             WHERE info_hash = ?10",
            rusqlite::params![
                progress.total_size,
                progress.downloaded_bytes,
                progress.uploaded_bytes,
                progress.download_speed,
                progress.upload_speed,
                progress.progress,
                progress.peers,
                progress.seeds,
                progress.eta_seconds,
                progress.info_hash,
            ],
        )?;
        Ok(())
    }

    pub fn delete_download(conn: &rusqlite::Connection, info_hash: &str) -> Result<()> {
        conn.execute("DELETE FROM downloads WHERE info_hash = ?1", [info_hash])?;
        Ok(())
    }
}

