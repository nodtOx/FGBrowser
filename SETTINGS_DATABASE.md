# Settings Storage: Moved to SQLite Database

## Overview

Application settings are now stored in the SQLite database instead of using Tauri's Store plugin. This provides better integration, simpler architecture, and eliminates an unnecessary dependency.

## Changes Made

### 1. Database Schema

**New table: `settings`**

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)
```

Settings are stored as JSON in the `value` column with key `'app_settings'`.

### 2. Rust Implementation

**New struct in `database.rs`:**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    // General
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub notifications: bool,

    // Download
    pub download_path: String,
    pub max_simultaneous_downloads: i32,
    pub auto_start_downloads: bool,
    pub seed_after_complete: bool,
    pub seed_ratio: f64,

    // Network
    pub max_download_speed: i32,
    pub max_upload_speed: i32,
    pub port: i32,
    pub use_upnp: bool,
    pub use_dht: bool,

    // Appearance
    pub font_size: i32,
    pub compact_mode: bool,
    pub show_thumbnails: bool,
    pub animations_enabled: bool,

    // Database
    pub db_path: String,
    pub auto_refresh: bool,
    pub refresh_interval: i32,
}
```

**New methods in `database.rs`:**

- `init_settings_table()` - Creates settings table
- `get_settings()` - Loads settings from database
- `save_settings(&settings)` - Saves settings to database

**New Tauri commands in `commands.rs`:**

- `get_settings()` - Exposed to frontend
- `save_settings(settings)` - Exposed to frontend

### 3. Frontend Implementation

**Updated `Settings.svelte`:**

- Removed `@tauri-apps/plugin-store` dependency
- Uses `invoke('get_settings')` to load
- Uses `invoke('save_settings', { settings })` to save
- Auto-saves with 500ms debounce (same as before)

### 4. Removed Dependencies

**From `Cargo.toml`:**

```toml
- tauri-plugin-store = "2"
```

**From `package.json`:**

```json
- "@tauri-apps/plugin-store": "^2"
```

**From `lib.rs`:**

```rust
- .plugin(tauri_plugin_store::Builder::new().build())
```

## Benefits

### 1. **Simpler Architecture**

- One database for everything (games + settings)
- No need to manage multiple storage systems
- Easier to understand and maintain

### 2. **Better Integration**

- Settings live alongside app data
- Can use SQL queries if needed
- Easier backup (single database file)

### 3. **Fewer Dependencies**

- Removed `tauri-plugin-store` dependency
- Smaller bundle size
- Faster compilation

### 4. **Easier Migration**

- Export entire database (including settings)
- Single backup file
- Settings migrate with data

### 5. **Type Safety**

- Strong typing on both Rust and TypeScript sides
- Compile-time checking
- Better IDE support

## Storage Location

Settings are stored in the same database as game data:

- **Development**: `../repacks.db`
- **Production**: App-specific location

Settings table structure:

```
┌─────────────┬───────────────────────────┬────────────────────┐
│ key         │ value                     │ updated_at         │
├─────────────┼───────────────────────────┼────────────────────┤
│ app_settings│ {"auto_start":false,...}  │ 2025-01-14 10:30:00│
└─────────────┴───────────────────────────┴────────────────────┘
```

## How It Works

### Loading Settings (Startup)

1. Component mounts
2. Calls `invoke('get_settings')`
3. Rust reads from database
4. Deserializes JSON to `AppSettings`
5. Returns to frontend
6. Frontend populates UI

### Saving Settings (Auto-save)

1. User changes a setting
2. Svelte reactive statement triggers
3. Debounce timer waits 500ms
4. Calls `invoke('save_settings', { settings })`
5. Rust serializes to JSON
6. Stores in database with timestamp

### Default Values

If settings don't exist in database:

- Rust returns `AppSettings::default()`
- Frontend uses `??` operators for fallbacks
- Download path uses system's Downloads folder

## Migration from Store Plugin

### Automatic Migration

Settings automatically migrate on first run:

1. Old Tauri Store settings are **not** imported (clean start)
2. Default values are used
3. User settings saved to database immediately
4. No manual migration needed

### Manual Migration (Optional)

If you want to preserve old Store settings:

1. Export old `settings.json` from Tauri Store location
2. Convert JSON keys from snake_case to TypeScript format
3. Manually set values in UI
4. Settings auto-save to database

**Store file location:**

- Windows: `%APPDATA%\com.pc.desktop\settings.json`
- macOS: `~/Library/Application Support/com.pc.desktop/settings.json`
- Linux: `~/.config/com.pc.desktop/settings.json`

## Testing

### Verify Settings Persist

1. Open Settings page
2. Change some settings
3. Close and reopen app
4. Verify settings persisted

### Verify Database Storage

```bash
sqlite3 repacks.db
> SELECT * FROM settings;
```

Should show JSON blob with all settings.

### Verify Auto-save

1. Change a setting
2. Wait 500ms
3. Check database (should update)
4. No need to click "Save"

## API Reference

### Rust Side

```rust
// Get settings
let settings = db.get_settings()?;

// Save settings
db.save_settings(&settings)?;

// Initialize table
db.init_settings_table()?;
```

### Frontend Side

```typescript
import { invoke } from '@tauri-apps/api/core';

// Load settings
const settings = await invoke('get_settings');

// Save settings
await invoke('save_settings', {
  settings: {
    auto_start: false,
    download_path: '/path/to/downloads',
    // ... other fields
  },
});
```

## Future Enhancements

- [ ] Settings export/import (JSON)
- [ ] Settings reset to factory defaults
- [ ] Settings profiles (multiple configurations)
- [ ] Settings versioning/migration
- [ ] Settings validation
- [ ] Cloud sync support

## Troubleshooting

### Settings Not Persisting

- Check database file exists and is writable
- Check console for errors
- Verify `settingsLoaded` flag is true

### Settings Reset to Default

- Settings table may not exist (will auto-create)
- Database file may be recreated
- Check for database migration issues

### Type Errors

TypeScript may show errors about Store types:

1. Restart TypeScript server
2. Run `npm install` in desktop folder
3. Reload VS Code

## Summary

Settings management is now fully integrated with the SQLite database, providing a cleaner, more maintainable solution with fewer dependencies. All settings auto-save with the same user experience, but now leverage the existing database infrastructure.

**Key Points:**

- ✅ Settings stored in SQLite database
- ✅ No Tauri Store plugin needed
- ✅ Auto-save with 500ms debounce
- ✅ Type-safe on both ends
- ✅ Easier backup and migration
- ✅ Single storage location
