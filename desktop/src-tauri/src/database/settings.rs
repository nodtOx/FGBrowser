use rusqlite::Result;
use super::models::AppSettings;

pub struct SettingsQueries;

impl SettingsQueries {
    pub fn init_settings_table(conn: &rusqlite::Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    pub fn get_settings(conn: &rusqlite::Connection) -> Result<AppSettings> {
        // Ensure settings table exists
        let _ = Self::init_settings_table(conn);

        // Try to get settings from database
        match conn.query_row(
            "SELECT value FROM settings WHERE key = 'app_settings'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(json_str) => {
                // Parse JSON
                serde_json::from_str(&json_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Return default settings if not found
                Ok(AppSettings::default())
            }
            Err(e) => Err(e),
        }
    }

    pub fn save_settings(conn: &rusqlite::Connection, settings: &AppSettings) -> Result<()> {
        // Ensure settings table exists
        Self::init_settings_table(conn)?;

        // Serialize to JSON
        let json_str = serde_json::to_string(settings)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Insert or replace
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) 
             VALUES ('app_settings', ?1, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET 
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP",
            [&json_str],
        )?;

        Ok(())
    }
}

